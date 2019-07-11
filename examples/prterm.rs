extern crate term;

fn main() {
    // First, print a description of the current terminal
    let terminfo = term::terminfo::TermInfo::from_env();
    dbg!(&terminfo);

    // Secondly, try using the methods that rustup needs.  If the output looks
    // correct then the terminal really works.
    let mut t = term::stdout().unwrap();

    if let Ok(_) = t.attr(term::Attr::Bold) {
        writeln!(t, "Bold!").unwrap();
        t.reset().unwrap();
    } else {
        writeln!(t, "Bold not supported").unwrap();
    }

    if t.supports_color() {
        t.fg(term::color::GREEN).unwrap();
        writeln!(t, "Foreground green").unwrap();
        t.reset().unwrap();

        t.bg(term::color::GREEN).unwrap();
        writeln!(t, "Background green").unwrap();
        t.reset().unwrap();
    } else {
        writeln!(t, "Color not supported").unwrap();
    }

    write!(t, "overwrite").unwrap();
    if let Ok(_) = t.carriage_return() {
        writeln!(t, "overwrite supported").unwrap();
    } else {
        writeln!(t, " not supported").unwrap();
    }

    writeln!(t, "cursor up").unwrap();
    if let Ok(_) = t.cursor_up() {
        writeln!(t, "cursor up supported").unwrap();
    } else {
        writeln!(t, "          not supported").unwrap();
    }

    write!(t, "delete line").unwrap();
    if let Ok(_) = t.carriage_return() {
        if let Ok(_) = t.delete_line() {
            writeln!(t, "delete line supported").unwrap();
        } else {
            writeln!(t, " not supported").unwrap();
        }
    } else {
        writeln!(t, " is pointless without carriage return").unwrap();
    }

    t.reset().unwrap();
}
