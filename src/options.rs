use clap::{App, Arg};
use bindgen::{Builder, CodegenConfig, builder};
use std::fs::File;
use std::io::{self, Error, ErrorKind};

/// Construct a new [`Builder`](./struct.Builder.html) from command line flags.
pub fn builder_from_flags<I>(args: I)
                             -> Result<(Builder, Box<io::Write>), io::Error>
    where I: Iterator<Item = String>,
{
    let matches = App::new("bindgen")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Generates Rust bindings from C/C++ headers.")
        .usage("bindgen [FLAGS] [OPTIONS] <header> -- <clang-args>...")
        .args(&[
            Arg::with_name("header")
                .help("C or C++ header file")
                .required(true),
            Arg::with_name("bitfield-enum")
                .long("bitfield-enum")
                .help("Mark any enum whose name matches <regex> as a set of \
                       bitfield flags instead of an enumeration.")
                .value_name("regex")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("constified-enum")
                .long("constified-enum")
                .help("Mark any enum whose name matches <regex> as a set of \
                       constants instead of an enumeration.")
                .value_name("regex")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("blacklist-type")
                .long("blacklist-type")
                .help("Mark a type as hidden.")
                .value_name("type")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("no-derive-debug")
                .long("no-derive-debug")
                .help("Avoid deriving Debug on any type."),
            Arg::with_name("builtins")
                .long("builtins")
                .help("Output bindings for builtin definitions, e.g. \
                       __builtin_va_list."),
            Arg::with_name("ctypes-prefix")
                .long("ctypes-prefix")
                .help("Use the given prefix before raw types instead of \
                      ::std::os::raw.")
                .value_name("prefix")
                .takes_value(true),
            // All positional arguments after the end of options marker, `--`
            Arg::with_name("clang-args")
                .multiple(true),
            Arg::with_name("dummy-uses")
                .long("dummy-uses")
                .help("For testing purposes, generate a C/C++ file containing \
                       dummy uses of all types defined in the input header.")
                .takes_value(true),
            Arg::with_name("emit-clang-ast")
                .long("emit-clang-ast")
                .help("Output the Clang AST for debugging purposes."),
            Arg::with_name("emit-ir")
                .long("emit-ir")
                .help("Output our internal IR for debugging purposes."),
            Arg::with_name("enable-cxx-namespaces")
                .long("enable-cxx-namespaces")
                .help("Enable support for C++ namespaces."),
            Arg::with_name("disable-name-namespacing")
                .long("disable-name-namespacing")
                .help("Disable name namespacing if namespaces are disabled."),
            Arg::with_name("framework")
                .long("framework-link")
                .help("Link to framework.")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("ignore-functions")
                .long("ignore-functions")
                .help("Do not generate bindings for functions or methods. This \
                       is useful when you only care about struct layouts."),
            Arg::with_name("generate")
                .long("generate")
                .help("Generate a given kind of items, split by commas. \
                       Valid values are \"functions\",\"types\", \"vars\" and \
                       \"methods\".")
                .takes_value(true),
            Arg::with_name("ignore-methods")
                .long("ignore-methods")
                .help("Do not generate bindings for methods."),
            Arg::with_name("dynamic")
                .short("l")
                .long("link")
                .help("Link to dynamic library.")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("no-convert-floats")
                .long("no-convert-floats")
                .help("Don't automatically convert floats to f32/f64."),
            Arg::with_name("no-unstable-rust")
                .long("no-unstable-rust")
                .help("Do not generate unstable Rust code.")
                .multiple(true), // FIXME: Pass legacy test suite
            Arg::with_name("opaque-type")
                .long("opaque-type")
                .help("Mark a type as opaque.")
                .value_name("type")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Write Rust bindings to <output>.")
                .takes_value(true),
            Arg::with_name("raw-line")
                .long("raw-line")
                .help("Add a raw line of Rust code at the beginning of output.")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("static")
                .long("static-link")
                .help("Link to static library.")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("use-core")
                .long("use-core")
                .help("Use types from Rust core instead of std."),
            Arg::with_name("conservative-inline-namespaces")
                .long("conservative-inline-namespaces")
                .help("Conservatively generate inline namespaces to avoid name \
                       conflicts."),
            Arg::with_name("use-msvc-mangling")
                .long("use-msvc-mangling")
                .help("MSVC C++ ABI mangling. DEPRECATED: Has no effect."),
            Arg::with_name("whitelist-function")
                .long("whitelist-function")
                .help("Whitelist all the free-standing functions matching \
                       <regex>. Other non-whitelisted functions will not be \
                       generated.")
                .value_name("regex")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("whitelist-type")
                .long("whitelist-type")
                .help("Whitelist the type. Other non-whitelisted types will \
                       not be generated.")
                .value_name("type")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
            Arg::with_name("whitelist-var")
                .long("whitelist-var")
                .help("Whitelist all the free-standing variables matching \
                       <regex>. Other non-whitelisted variables will not be \
                       generated.")
                .value_name("regex")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
        ]) // .args()
        .get_matches_from(args);

    let mut builder = builder();

    if let Some(header) = matches.value_of("header") {
        builder = builder.header(header);
    } else {
        return Err(Error::new(ErrorKind::Other, "Header not found"));
    }

    if let Some(bitfields) = matches.values_of("bitfield-enum") {
        for regex in bitfields {
            builder = builder.bitfield_enum(regex);
        }
    }

    if let Some(bitfields) = matches.values_of("constified-enum") {
        for regex in bitfields {
            builder = builder.constified_enum(regex);
        }
    }

    if let Some(hidden_types) = matches.values_of("blacklist-type") {
        for ty in hidden_types {
            builder = builder.hide_type(ty);
        }
    }

    if matches.is_present("builtins") {
        builder = builder.emit_builtins();
    }

    if matches.is_present("no-derive-debug") {
        builder = builder.derive_debug(false);
    }

    if let Some(prefix) = matches.value_of("ctypes-prefix") {
        builder = builder.ctypes_prefix(prefix);
    }

    if let Some(dummy) = matches.value_of("dummy-uses") {
        builder = builder.dummy_uses(dummy);
    }

    if let Some(links) = matches.values_of("dynamic") {
        for library in links {
            builder = builder.link(library);
        }
    }

    if let Some(what_to_generate) = matches.value_of("generate") {
        let mut config = CodegenConfig::nothing();
        for what in what_to_generate.split(",") {
            match what {
                "functions" => config.functions = true,
                "types" => config.types = true,
                "vars" => config.vars = true,
                "methods" => config.methods = true,
                _ => {
                    return Err(Error::new(ErrorKind::Other,
                                          "Unknown generate item"));
                }
            }
        }
        builder = builder.with_codegen_config(config);
    }

    if matches.is_present("emit-clang-ast") {
        builder = builder.emit_clang_ast();
    }

    if matches.is_present("emit-ir") {
        builder = builder.emit_ir();
    }

    if matches.is_present("enable-cxx-namespaces") {
        builder = builder.enable_cxx_namespaces();
    }

    if matches.is_present("disable-name-namespacing") {
        builder = builder.disable_name_namespacing();
    }

    if let Some(links) = matches.values_of("framework") {
        for framework in links {
            builder = builder.link_framework(framework);
        }
    }

    if matches.is_present("ignore-functions") {
        builder = builder.ignore_functions();
    }

    if matches.is_present("ignore-methods") {
        builder = builder.ignore_methods();
    }

    if matches.is_present("no-unstable-rust") {
        builder = builder.no_unstable_rust();
    }

    if matches.is_present("no-convert-floats") {
        builder = builder.no_convert_floats();
    }

    if let Some(opaque_types) = matches.values_of("opaque-type") {
        for ty in opaque_types {
            builder = builder.opaque_type(ty);
        }
    }

    if let Some(lines) = matches.values_of("raw-line") {
        for line in lines {
            builder = builder.raw_line(line);
        }
    }

    if let Some(links) = matches.values_of("static") {
        for library in links {
            builder = builder.link_static(library);
        }
    }

    if matches.is_present("use-core") {
        builder = builder.use_core();
    }

    if matches.is_present("conservative-inline-namespaces") {
        builder = builder.conservative_inline_namespaces();
    }

    if let Some(whitelist) = matches.values_of("whitelist-function") {
        for regex in whitelist {
            builder = builder.whitelisted_function(regex);
        }
    }

    if let Some(whitelist) = matches.values_of("whitelist-type") {
        for regex in whitelist {
            builder = builder.whitelisted_type(regex);
        }
    }

    if let Some(whitelist) = matches.values_of("whitelist-var") {
        for regex in whitelist {
            builder = builder.whitelisted_var(regex);
        }
    }

    if let Some(args) = matches.values_of("clang-args") {
        for arg in args {
            builder = builder.clang_arg(arg);
        }
    }

    let output = if let Some(path) = matches.value_of("output") {
        let file = try!(File::create(path));
        Box::new(io::BufWriter::new(file)) as Box<io::Write>
    } else {
        Box::new(io::BufWriter::new(io::stdout())) as Box<io::Write>
    };

    Ok((builder, output))
}
