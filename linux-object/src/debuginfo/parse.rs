use object::{Object, ObjectSection, SymbolMap, SymbolMapName};
use addr2line::{Context, Location};
use typed_arena::Arena;
use fallible_iterator::FallibleIterator;

fn print_loc(loc: Option<&Location>, basenames: bool, llvm: bool) {
    if let Some(ref loc) = loc {
        if let Some(ref file) = loc.file.as_ref() {
            let path = if basenames {
                Path::new(Path::new(file).file_name().unwrap())
            } else {
                Path::new(file)
            };
            print!("{}:", path.display());
        } else {
            print!("??:");
        }
        if llvm {
            print!("{}:{}", loc.line.unwrap_or(0), loc.column.unwrap_or(0));
        } else if let Some(line) = loc.line {
            print!("{}", line);
        } else {
            print!("?");
        }
        println!();
    } else if llvm {
        println!("??:0:0");
    } else {
        println!("??:0");
    }
}

fn print_function(name: Option<&str>, language: Option<gimli::DwLang>, demangle: bool) {
    if let Some(name) = name {
        if demangle {
            print!("{}", addr2line::demangle_auto(Cow::from(name), language));
        } else {
            print!("{}", name);
        }
    } else {
        print!("??");
    }
}

fn load_file_section<'input, 'arena, Endian: gimli::Endianity>(
    id: gimli::SectionId,
    file: &object::File<'input>,
    endian: Endian,
    arena_data: &'arena Arena<Cow<'input, [u8]>>,
) -> Result<gimli::EndianSlice<'arena, Endian>, ()> {
    // TODO: Unify with dwarfdump.rs in gimli.
    let name = id.name();
    match file.section_by_name(name) {
        Some(section) => match section.uncompressed_data().unwrap() {
            Cow::Borrowed(b) => Ok(gimli::EndianSlice::new(b, endian)),
            Cow::Owned(b) => Ok(gimli::EndianSlice::new(arena_data.alloc(b.into()), endian)),
        },
        None => Ok(gimli::EndianSlice::new(&[][..], endian)),
    }
}

fn find_name_from_symbols<'a>(
    symbols: &'a SymbolMap<SymbolMapName>,
    probe: u64,
) -> Option<&'a str> {
    symbols.get(probe).map(|x| x.name())
}

pub fn parse_zcore_elf(zcore: Vec<u8>) {
    let object = &object::File::parse(zcore).unwrap();
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    let arena_data = Arena::new();
    let mut load_section = |id: gimli::SectionId| -> Result<_, _> {
        load_file_section(id, object, endian, &arena_data)
    };
    let symbols = object.symbol_map();
    let dwarf = gimli::Dwarf::load(&mut load_section).unwrap();

    let ctx = Context::from_dwarf(dwarf).unwrap();

    let probe = Some(0xFFFFFFC08028F912);
    let demangle = true;
    let basenames = false;
    let llvm = true;
    let mut printed_anything = false;
    if let Some(probe) = probe {
        let mut frames = ctx.find_frames(probe).unwrap().enumerate();
        while let Some((i, frame)) = frames.next().unwrap() {
            if i != 0 {
                print!(" (inlined by) ");
            }

            if let Some(func) = frame.function {
                print_function(
                    func.raw_name().ok().as_ref().map(AsRef::as_ref),
                    func.language,
                    demangle,
                );
            } else {
                let name = find_name_from_symbols(&symbols, probe);
                print_function(name, None, demangle);
            }

            print!(" at ");
            print_loc(frame.location.as_ref(), basenames, llvm);
            printed_anything = true;
        }
    }

    if !printed_anything {
        let name = probe.and_then(|probe| find_name_from_symbols(&symbols, probe));
        print_function(name, None, demangle);
        print!(" at ");
        print_loc(None, basenames, llvm);
    }
    println!();
}
