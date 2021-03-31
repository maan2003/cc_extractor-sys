use std::env;
use std::path::PathBuf;

fn main() {
    utf8proc();
    // use cmake to build lib_ccx
    let lib = cmake::build("ccextractor/src");

    println!("cargo:rustc-link-search=native={}/lib", lib.display());
    println!("cargo:rustc-link-lib=static=ccx");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .whitelist_type("^ccx_.*")
        .whitelist_function("^ccx_.*")
        .whitelist_var("^ccx_.*")
        .blacklist_item("ccx_options")
        .blacklist_function("print_end_msg")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    for fn_name in WHITELIST_FUNCTIONS.iter().copied() {
        builder = builder.whitelist_function(fn_name);
    }

    for var in WHITELIST_VARS.iter().copied() {
        builder = builder.whitelist_var(var);
    }

    for enum_name in BITFEILD_ENUMS.iter().copied() {
        builder = builder.bitfield_enum(enum_name);
    }

    for enum_name in RUSTIFIED_ENUMS.iter().copied() {
        builder = builder.rustified_enum(enum_name);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn utf8proc() {
    cc::Build::new()
        .files(&["ccextractor/src/thirdparty/utf8proc/utf8proc.c"])
        .compile("utf8proc");
}

// FIXME: using freetype-sys for now, this give pages of errors and warnings
// fn freetype() {
//     let files = [
//         "autofit/autofit.c",
//         "base/ftbase.c",
//         "base/ftbbox.c",
//         "base/ftbdf.c",
//         "base/ftbitmap.c",
//         "base/ftcid.c",
//         "base/ftfntfmt.c",
//         "base/ftfstype.c",
//         "base/ftgasp.c",
//         "base/ftglyph.c",
//         "base/ftgxval.c",
//         "base/ftinit.c",
//         "base/ftlcdfil.c",
//         "base/ftmm.c",
//         "base/ftotval.c",
//         "base/ftpatent.c",
//         "base/ftpfr.c",
//         "base/ftstroke.c",
//         "base/ftsynth.c",
//         "base/ftsystem.c",
//         "base/fttype1.c",
//         "base/ftwinfnt.c",
//         "bdf/bdf.c",
//         "bzip2/ftbzip2.c",
//         "cache/ftcache.c",
//         "cff/cff.c",
//         "cid/type1cid.c",
//         "gzip/ftgzip.c",
//         "lzw/ftlzw.c",
//         "pcf/pcf.c",
//         "pfr/pfr.c",
//         "psaux/psaux.c",
//         "pshinter/pshinter.c",
//         "psnames/psnames.c",
//         "raster/raster.c",
//         "sfnt/sfnt.c",
//         "smooth/smooth.c",
//         "truetype/truetype.c",
//         "type1/type1.c",
//         "type42/type42.c",
//         "winfonts/winfnt.c",
//     ];
//     let freetype_root = Path::new("ccextractor/src/thirdparty/freetype");
//     cc::Build::new()
//         .files(files.iter().map(|path| freetype_root.join(path)))
//         .include(freetype_root.join("include"))
//         .compile("freetype");
// }

// TODO: these should be in a seperate config file
// TODO: there are more public functions probably.
const WHITELIST_FUNCTIONS: &[&str] = &[
    "init_libraries",
    "dinit_libraries",
    "parse_parameters",
    "print_usage",
    "atoi_hex",
    "stringztoms",
    "position_sanity_check",
    "init_file_buffer",
    "ps_get_more_data",
    "general_get_more_data",
    "position_sanity_check",
    "init_file_buffer",
    "ps_get_more_data",
    "general_get_more_data",
    "raw_loop",
    "process_raw",
    "general_loop",
    "process_hex",
    "rcwt_loop",
    "ccx_mxf_getmoredata",
    "asf_get_more_data",
    "wtv_get_more_data",
    "process_m2v",
    "writercwtdata",
    "flushbuffer",
    "writeraw",
    "init_write",
    "temporarily_close_output",
    "temporarily_open_output",
    "dinit_write",
    "read_video_pes_header",
    "init_ts",
    "ts_readpacket",
    "ts_readstream",
    "ts_get_more_data",
    "write_section",
    "ts_buffer_psi_packet",
    "parse_PMT",
    "parse_PAT",
    "parse_EPG_packet",
    "EPG_free",
    "EPG_DVB_decode_string",
    "parse_SDT",
    "get_video_stream",
    "myth_loop",
    "matroska_loop",
    "fatal",
    "sleep_secs",
    "dump",
    "in_array",
    "hex_to_int",
    "hex_string_to_int",
    "timestamp_to_srttime",
    "levenshtein_dist",
    "millis_to_date",
    "signal_handler",
    "change_filename",
    "m_signal",
    "buffered_seek",
    "build_parity_table",
    "tlt_process_pes_packet",
    "telxcc_init",
    "telxcc_close",
    "tlt_read_rcwt",
    "telxcc_configure",
    "telxcc_update_gt",
    "is_decoder_processed_enough",
    "update_decoder_list_cinfo",
    "update_decoder_list",
    "update_encoder_list_cinfo",
    "update_encoder_list",
    "get_encoder_by_pn",
    "detect_myth",
    "detect_stream_type",
    "isValidMP4Box",
    "writercwtdata",
    "print_file_report",
    "params_dump",
    "process_hdcc",
    "anchor_hdcc",
    "store_hdcc",
    "init_hdcc",
    "return_to_buffer",
    "switch_to_next_file",
    "close_input_file",
    "close_input_file",
    "prepare_for_new_file",
    "get_total_file_size",
    "get_file_size",
    "user_data",
    "hardsubx",
    "processmp4",
    "dumpchapters",
    "mprint",
    "print_mstime_static",
    "init_options",
    "parse_configuration",
];

const BITFEILD_ENUMS: &[&str] = &["ccx_debug_message_types", "ccx_output_format"];

const RUSTIFIED_ENUMS: &[&str] = &[
    "ccx_stream_mode",
    "ccx_stream_type",
    "ccx_avc_nal_types",
    "ccx_dtvcc_pen_text_tag",
    "cc_modes",
    "font_bits",
    "ccx_decoder_608_color_code",
    "ccx_eia608_format",
    "ccx_bufferdata_type",
    "ccx_dtvcc_window_pd",
    "ccx_code_type",
    "ccx_common_logging_gui",
    "bool_t",
    "ccx_datasource",
    "ccx_dtvcc_pen_anchor_point",
    "ccx_dtvcc_pen_edge",
    "ccx_dtvcc_pen_font_style",
    "ccx_dtvcc_pen_edge",
    "ccx_dtvcc_pen_font_style",
    "ccx_dtvcc_pen_offset",
    "ccx_dtvcc_pen_size",
    "ccx_dtvcc_window_border",
    "ccx_dtvcc_window_ed",
    "ccx_dtvcc_window_justify",
    "ccx_dtvcc_window_fo",
    "ccx_dtvcc_window_sd",
    "ccx_dtvcc_window_sde",
    "ccx_encoding_type",
    "ccx_frame_type",
    "ccx_mpeg_descriptor",
    "ccx_output_date_format",
    "ccx_stream_mode_enum",
    "subtype",
    "subdatatype",
];

const WHITELIST_VARS: &[&str] = &[
    "tlt_config",
    "current_fps",
    "cb_field1",
    "cb_field2",
    "cb_708",
    "pts_big_change",
    "gop_time",
    "first_gop_time",
    "pts_big_change",
    "MPEG_CLOCK_FREQ",
    "EXIT_NO_CAPTIONS",
    "EXIT_OK",
    "EXIT_NO_INPUT_FILES",
    "EXIT_WITH_HELP",
];
