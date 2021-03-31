//! This is a line by line port so it doesn't try to refactor the code

// FIXME: using freetype_sys's freetype should be using our vendored one.
extern crate freetype_sys;

use std::convert::TryInto;
use std::env;
use std::ffi::CString;
use std::mem::{self, MaybeUninit};
use std::os::raw::{c_int, c_longlong};
use std::process;

// I guess use cc_extractor_sys::* is better
use cc_extractor_sys::{
    cb_708, cb_field1, cb_field2, ccx_common_timing_settings, ccx_list_head_to_cc_decode,
    ccx_output_format, ccx_s_options, ccx_stream_mode_enum, close_input_file, current_fps,
    dinit_libraries, dumpchapters, first_gop_time, general_loop, get_total_file_size, gop_time,
    init_libraries, init_options, is_decoder_processed_enough, matroska_loop, mprint, myth_loop,
    params_dump, parse_configuration, parse_parameters, prepare_for_new_file, print_file_report,
    print_mstime_static, print_usage, processmp4, pts_big_change, raw_loop, switch_to_next_file,
    tlt_config, EXIT_NO_CAPTIONS, EXIT_NO_INPUT_FILES, EXIT_OK, EXIT_WITH_HELP, MPEG_CLOCK_FREQ,
};

#[no_mangle]
pub static mut ccx_options: MaybeUninit<ccx_s_options> = MaybeUninit::uninit();

#[no_mangle]
pub static mut terminate_asap: c_int = 0;

// TODO: add signal handlers
#[no_mangle]
pub extern "C" fn print_end_msg() {
    println!("Goodbye from Rust!");
}

// TODO: convert feature flags to cargo features
unsafe fn api_start(mut api_options: ccx_s_options) -> i32 {
    // let ctx = ptr::null_mut();
    // let de_ctx = ptr::null_mut();
    let mut ret = 0;
    let mut tmp;
    let mut stream_mode = ccx_stream_mode_enum::CCX_SM_ELEMENTARY_OR_NOT_FOUND;
    let mut ctx = init_libraries(&mut api_options);
    if ctx.is_null() {
        // TODO: provide good error messages, possibly use anyhow
        panic!("Something bad happened");
    }

    let mut show_myth_banner = false;
    params_dump(ctx);
    // default teletext page
    if tlt_config.page > 0 {
        // dec to BCD, magazine pages numbers are in BCD (ETSI 300 706)
        tlt_config.page = ((tlt_config.page / 100) << 8)
            | (((tlt_config.page / 10) % 10) << 4)
            | (tlt_config.page % 10);
    }

    if api_options.transcript_settings.xds != 0 {
        if api_options.write_format != ccx_output_format::CCX_OF_TRANSCRIPT {
            api_options.transcript_settings.xds = 0;
            // using println for, show probably be using log or tracing
            println!(
                "Warning: -xds ignored, XDS can only be exported to transcripts at this time.\n",
            );
        }
    }
    if api_options.binary_concat != 0 {
        (*ctx).total_inputsize = get_total_file_size(ctx);
        if (*ctx).total_inputsize < 0 {
            // TODO: provide good error messages
            panic!("Failed to open input file");
        }
    }
    // TODO: handle signals
    // signal_ctx = ctx;
    // m_signal(SIGINT, sigint_handler);
    // m_signal(SIGTERM, sigterm_handler);
    // m_signal(SIGUSR1, sigusr1_handler);
    while switch_to_next_file(ctx, 0) != 0 {
        prepare_for_new_file(ctx);
        // SAFETY: get stream mode will always return a stream mode
        stream_mode = mem::transmute(((*(*ctx).demux_ctx).get_stream_mode.unwrap())(
            (*ctx).demux_ctx,
        ));

        // Disable sync check for raw formats - they have the right timeline.
        // Also true for bin formats, but -nosync might have created a
        // broken timeline for debug purposes.
        // Disable too in MP4, specs doesn't say that there can't be a jump
        use ccx_stream_mode_enum::*;
        if let CCX_SM_MCPOODLESRAW | CCX_SM_RCWT | CCX_SM_MP4 = stream_mode {
            ccx_common_timing_settings.disable_sync_check = 1;
        }
        /* -----------------------------------------------------------------
        MAIN LOOP
        ----------------------------------------------------------------- */
        match stream_mode {
            CCX_SM_ELEMENTARY_OR_NOT_FOUND => {
                // If !0 then the user selected something
                if api_options.use_gop_as_pts == 0 {
                    api_options.use_gop_as_pts = 1; // Force GOP timing for ES
                }
                ccx_common_timing_settings.is_elementary_stream = 1;
            }

            CCX_SM_TRANSPORT | CCX_SM_PROGRAM | CCX_SM_ASF | CCX_SM_WTV | CCX_SM_GXF
            | CCX_SM_MXF => {
                // If !0 then the user selected something
                if api_options.use_gop_as_pts == 0 {
                    api_options.use_gop_as_pts = 0;
                }
                if api_options.ignore_pts_jumps != 0 {
                    ccx_common_timing_settings.disable_sync_check = 1;
                }
                mprint!("\rAnalyzing data in general mode\n");
                tmp = general_loop(ctx);
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_MCPOODLESRAW => {
                mprint!("\rAnalyzing data in McPoodle raw mode\n");
                tmp = raw_loop(ctx);
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_RCWT => {
                mprint!("\rAnalyzing data in CCExtractor's binary format\n");
                tmp = raw_loop(ctx);
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_MYTH => {
                mprint!("\rAnalyzing data in MythTV mode\n");
                show_myth_banner = true;
                tmp = myth_loop(ctx);
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_MP4 => {
                mprint!("\rAnalyzing data with GPAC (MP4 library)\n");
                close_input_file(ctx); // No need to have it open. GPAC will do it for us
                if (*ctx).current_file == -1 {
                    // We don't have a file to open, must be stdin, and GPAC is incompatible with stdin
                    panic!("MP4 requires an actual file, it's not possible to read from a stream, including stdin.\n");
                }
                if api_options.extract_chapters != 0 {
                    tmp = dumpchapters(
                        ctx,
                        &mut (*ctx).mp4_cfg,
                        *(*ctx).inputfile.offset((*ctx).current_file as _),
                    );
                } else {
                    tmp = processmp4(
                        ctx,
                        &mut (*ctx).mp4_cfg,
                        *(*ctx).inputfile.offset((*ctx).current_file as _),
                    );
                }
                if api_options.print_file_reports != 0 {
                    print_file_report(ctx);
                }
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_MKV => {
                mprint!("\rAnalyzing data in Matroska mode\n");
                tmp = matroska_loop(ctx);
                if ret == 0 {
                    ret = tmp;
                }
            }
            CCX_SM_AUTODETECT => {
                unreachable!("Cannot be reached!");
            }
        }
    }

    let head: *mut _ = &mut (*ctx).dec_ctx_head;
    let mut curr = (*head).next;
    while curr != head {
        let dec_ctx = ccx_list_head_to_cc_decode(curr);
        mprint!("\n");
        // TODO: add debugging
        // Add one frame as fts_max marks the beginning of the last frame,
        // but we need the end.
        (*(*dec_ctx).timing).fts_global +=
            (*(*dec_ctx).timing).fts_max + ((1000.0 / current_fps) as c_longlong);
        // CFS: At least in Hauppage mode, cb_field can be responsible for ALL the
        // timing (cb_fields having a huge number and fts_now and fts_global being 0 all
        // the time), so we need to take that into account in fts_global before resetting
        // counters.
        if cb_field1 != 0 {
            (*(*dec_ctx).timing).fts_global += (cb_field1 * 1001 / 3) as i64;
        } else if cb_field2 != 0 {
            (*(*dec_ctx).timing).fts_global += (cb_field2 * 1001 / 3) as i64;
        } else {
            (*(*dec_ctx).timing).fts_global += (cb_708 * 1001 / 3) as i64;
        }
        // Reset counters - This is needed if some captions are still buffered
        // and need to be written after the last file is processed.
        cb_field1 = 0;
        cb_field2 = 0;
        cb_708 = 0;
        (*(*dec_ctx).timing).fts_now = 0;
        (*(*dec_ctx).timing).fts_now = 0;
        if (*dec_ctx).total_pulldownframes != 0 {
            mprint!(
                "incl. pulldown frames:  %s  (%u frames at %.2ffps)\n",
                print_mstime_static(
                    (((*dec_ctx).total_pulldownframes * 1000) as f64 / current_fps) as c_longlong
                ),
                (*dec_ctx).total_pulldownframes,
                current_fps
            );
        }

        if (*(*dec_ctx).timing).pts_set >= 1 && (*(*dec_ctx).timing).min_pts != 0x01FFFFFFFF {
            let postsyncms =
                (((*dec_ctx).frames_since_last_gop * 1000) as f64 / current_fps) as c_longlong;
            mprint!(
                "\nMin PTS:				%s\n",
                print_mstime_static(
                    (*(*dec_ctx).timing).min_pts / (MPEG_CLOCK_FREQ / 1000) as i64
                        - (*(*dec_ctx).timing).fts_offset
                )
            );
            if pts_big_change != 0 {
                mprint!("(Reference clock was reset at some point, Min PTS is approximated)\n");
            }
            mprint!(
                "Max PTS:				%s\n",
                print_mstime_static(
                    (*(*dec_ctx).timing).sync_pts / (MPEG_CLOCK_FREQ / 1000) as i64 + postsyncms
                )
            );

            mprint!(
                "Length:				 %s\n",
                print_mstime_static(
                    (*(*dec_ctx).timing).sync_pts / (MPEG_CLOCK_FREQ / 1000) as i64 + postsyncms
                        - (*(*dec_ctx).timing).min_pts / (MPEG_CLOCK_FREQ / 1000) as i64
                        + (*(*dec_ctx).timing).fts_offset
                )
            );
        }

        // dvr-ms files have invalid GOPs
        if gop_time.inited != 0
            && first_gop_time.inited != 0
            && stream_mode != ccx_stream_mode_enum::CCX_SM_ASF
        {
            mprint!(
                "\nInitial GOP time:	   %s\n",
                print_mstime_static(first_gop_time.ms)
            );
            mprint!(
                "Final GOP time:		 %s%+3dF\n",
                print_mstime_static(gop_time.ms),
                (*dec_ctx).frames_since_last_gop
            );
            mprint!(
                "Diff. GOP length:	   %s%+3dF",
                print_mstime_static(gop_time.ms - first_gop_time.ms),
                (*dec_ctx).frames_since_last_gop
            );
            mprint!(
                "	(%s)\n\n",
                print_mstime_static(
                    gop_time.ms - first_gop_time.ms
                        + ((((*dec_ctx).frames_since_last_gop) * 1000) as f64 / 29.97)
                            as c_longlong
                )
            );
        }

        if (*dec_ctx).false_pict_header != 0 {
            mprint!(
                "Number of likely false picture headers (discarded): %d\n",
                (*dec_ctx).false_pict_header
            );
        }
        if (*dec_ctx).num_key_frames != 0 {
            mprint!("Number of key frames: %d\n", (*dec_ctx).num_key_frames);
        }

        if (*dec_ctx).stat_numuserheaders != 0 {
            mprint!(
                "Total user data fields: %d\n",
                (*dec_ctx).stat_numuserheaders
            );
        }
        if (*dec_ctx).stat_dvdccheaders != 0 {
            mprint!(
                "DVD-type user data fields: %d\n",
                (*dec_ctx).stat_dvdccheaders
            );
        }
        if ((*dec_ctx).stat_scte20ccheaders) != 0 {
            mprint!(
                "SCTE-20 type user data fields: %d\n",
                (*dec_ctx).stat_scte20ccheaders
            );
        }
        if ((*dec_ctx).stat_replay4000headers) != 0 {
            mprint!(
                "ReplayTV 4000 user data fields: %d\n",
                (*dec_ctx).stat_replay4000headers
            );
        }
        if ((*dec_ctx).stat_replay5000headers) != 0 {
            mprint!(
                "ReplayTV 5000 user data fields: %d\n",
                (*dec_ctx).stat_replay5000headers
            );
        }
        if ((*dec_ctx).stat_hdtv) != 0 {
            mprint!("HDTV type user data fields: %d\n", (*dec_ctx).stat_hdtv);
        }
        if (*dec_ctx).stat_dishheaders != 0 {
            mprint!(
                "Dish Network user data fields: %d\n",
                (*dec_ctx).stat_dishheaders
            );
        }
        if (*dec_ctx).stat_divicom != 0 {
            mprint!(
                "CEA608/Divicom user data fields: %d\n",
                (*dec_ctx).stat_divicom
            );

            mprint!("\n\nNOTE! The CEA 608 / Divicom standard encoding for closed\n");
            mprint!("caption is not well understood!\n\n");
            mprint!("Please submit samples to the developers.\n\n\n");
        }

        if is_decoder_processed_enough(ctx) == 1 {
            break;
        }
        curr = (*curr).next;
    }
    close_input_file(ctx);
    prepare_for_new_file(ctx); // To reset counters used by handle_end_of_data()

    // TODO: add performance data

    if is_decoder_processed_enough(ctx) == 1 {
        mprint!("\rNote: Processing was canceled before all data was processed because\n");
        mprint!("\rone or more user-defined limits were reached.\n");
    }
    dinit_libraries(&mut ctx);

    if ret == 0 {
        mprint!("\nNo captions were found in input.\n");
    }

    if show_myth_banner {
        mprint!(
            "NOTICE: Due to the major rework in 0.49, we needed to change part of the timing\n"
        );
        mprint!("code in the MythTV's branch. Please report results to the address above. If\n");
        mprint!("something is broken it will be fixed. Thanks\n");
    }
    if ret == 0 {
        EXIT_OK as i32
    } else {
        EXIT_NO_CAPTIONS as i32
    }
}
unsafe fn api_init_options() -> *mut ccx_s_options {
    init_options(ccx_options.as_mut_ptr());
    ccx_options.as_mut_ptr()
}

fn main() {
    unsafe {
        let mut args = env::args()
            .map(|arg| CString::new(arg).map(CString::into_raw))
            .collect::<Result<Vec<_>, _>>()
            .expect("Some argument contains null character");

        let argc = args.len().try_into().expect("Cannot cast argc into c_int");
        let argv = args.as_mut_ptr();
        let api_options = api_init_options();
        parse_configuration(api_options);
        // this requires *mut *mut c_char, but should be *const *const c_char
        let compile_ret = parse_parameters(
            api_options,
            argc,
            argv,
        );

        // drop the CStrings
        for ptr in args {
	    drop(CString::from_raw(ptr));
        }

        // we should be using match but `as i32` :(
        if compile_ret == EXIT_NO_INPUT_FILES as i32 {
            print_usage();
            process::exit(compile_ret);
        } else if compile_ret == EXIT_WITH_HELP as i32 {
            return;
        } else if compile_ret != EXIT_OK as i32 {
            process::exit(compile_ret);
        };
        let start_ret = api_start(*api_options);
        process::exit(start_ret);
    }
}
