mod helpers;

#[test]
fn colors() {
    helpers::fixture("colors");
}

#[test]
fn attrs() {
    helpers::fixture("attrs");
}

#[test]
fn attributes_formatted() {
    let mut parser = vt100::Parser::default();
    assert_eq!(parser.screen().attributes_formatted(), "\x1b[m");
    parser.process(b"\x1b[32mfoo\x1b[41mbar\x1b[33mbaz");
    assert_eq!(parser.screen().attributes_formatted(), "\x1b[m\x1b[33;41m");
    parser.process(b"\x1b[1m\x1b[39m");
    assert_eq!(parser.screen().attributes_formatted(), "\x1b[m\x1b[41;1m");
    parser.process(b"\x1b[m");
    assert_eq!(parser.screen().attributes_formatted(), "\x1b[m");
}

#[test]
fn underline_styles_colours_and_colon_forms() {
    let mut parser = vt100::Parser::default();
    let underline =
        |p: &vt100::Parser, col| p.screen().cell(0, col).unwrap().underline();

    // An underline style underlines, 4:0 stops it; so does 21.
    parser.process(b"\x1b[4:3ma\x1b[4:0mb\x1b[21mc\x1b[24md");
    assert!(underline(&parser, 0));
    assert!(!underline(&parser, 1));
    assert!(underline(&parser, 2));
    assert!(!underline(&parser, 3));

    // An underline colour, in either form, leaves the rest of its sequence.
    parser.process(b"\x1b[4;58;2;255;0;0;31me\x1b[58:5:9;59;24;32mf");
    assert!(underline(&parser, 4));
    assert_eq!(
        parser.screen().cell(0, 4).unwrap().fgcolor(),
        vt100::Color::Idx(1)
    );
    assert!(!underline(&parser, 5));
    assert_eq!(
        parser.screen().cell(0, 5).unwrap().fgcolor(),
        vt100::Color::Idx(2)
    );

    // RGB colours with colons, with or without the colour space.
    parser.process(b"\x1b[38:2::1:2:3mg\x1b[48:2:4:5:6mh");
    assert_eq!(
        parser.screen().cell(0, 6).unwrap().fgcolor(),
        vt100::Color::Rgb(1, 2, 3)
    );
    assert_eq!(
        parser.screen().cell(0, 7).unwrap().bgcolor(),
        vt100::Color::Rgb(4, 5, 6)
    );
}

#[test]
fn underline_styles_and_colours_are_kept() {
    use vt100::{Color, UnderlineStyle};
    let mut parser = vt100::Parser::default();
    parser.process(
        b"\x1b[4:3;58:2::255:0:0ma\x1b[21;58;5;4mb\x1b[4:5;59mc\x1b[24md\x1b[4me\x1b[0mf",
    );
    let kept = |p: &vt100::Parser| -> Vec<(UnderlineStyle, Color)> {
        (0..6)
            .map(|col| {
                let cell = p.screen().cell(0, col).unwrap();
                (cell.underline_style(), cell.underline_color())
            })
            .collect()
    };
    assert_eq!(
        kept(&parser),
        [
            (UnderlineStyle::Curly, Color::Rgb(255, 0, 0)),
            (UnderlineStyle::Double, Color::Idx(4)),
            (UnderlineStyle::Dashed, Color::Default),
            (UnderlineStyle::None, Color::Default),
            (UnderlineStyle::Single, Color::Default),
            (UnderlineStyle::None, Color::Default),
        ]
    );

    // Written out as tmux writes them, and read back the same.
    let formatted = parser.screen().contents_formatted();
    assert!(formatted.contains("4:3;58;2;255;0;0"), "{formatted:?}");
    let mut again = vt100::Parser::default();
    again.process(formatted.as_bytes());
    assert_eq!(kept(&again), kept(&parser));
}
