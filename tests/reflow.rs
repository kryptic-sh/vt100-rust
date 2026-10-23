mod helpers;

fn reflowing(rows: u16, cols: u16, history: usize) -> vt100::Parser {
    let mut parser = helpers::new(rows, cols, history);
    parser.screen_mut().set_reflow(true);
    parser
}

fn resize(parser: &mut vt100::Parser, rows: u16, cols: u16) {
    parser
        .screen_mut()
        .set_size(rows.try_into().unwrap(), cols.try_into().unwrap());
}

fn screen_rows(parser: &vt100::Parser) -> Vec<String> {
    let (_, cols) = parser.screen().size();
    parser.screen().rows(0, cols.get()).collect()
}

// The expected screens, histories and cursors are tmux 3.6's for the same
// output and sizes.

#[test]
fn a_wrapped_line_rewraps_to_the_new_width() {
    let mut parser = reflowing(4, 20, 10);
    parser.process(b"abcdefghijklmnopqrstuvwxyz0123\r\nx");
    resize(&mut parser, 4, 10);
    // The blank row below counts as a line: the first row goes into the
    // history.
    assert_eq!(screen_rows(&parser), ["klmnopqrst", "uvwxyz0123", "x", ""]);
    assert_eq!(parser.screen().scrollback_len(), 1);
    assert_eq!(parser.screen().cursor_position(), (2, 1));
    resize(&mut parser, 4, 40);
    assert_eq!(
        screen_rows(&parser),
        ["abcdefghijklmnopqrstuvwxyz0123", "x", "", ""]
    );
    assert_eq!(parser.screen().scrollback_len(), 0);
    assert_eq!(parser.screen().cursor_position(), (1, 1));
}

#[test]
fn a_cursor_past_its_lines_text_goes_to_its_end() {
    let mut parser = reflowing(3, 20, 0);
    parser.process(b"ab\x1b[6G");
    resize(&mut parser, 3, 10);
    assert_eq!(parser.screen().cursor_position(), (0, 2));
}

#[test]
fn the_history_rewraps_too_and_the_map_follows_the_lines() {
    let mut parser = reflowing(2, 10, 100);
    parser.process(b"0123456789abcdefghij\r\nline2\r\nline3\r\nline4");
    assert_eq!(parser.screen().scrollback_len(), 3);
    resize(&mut parser, 2, 20);
    assert_eq!(parser.screen().scrollback_len(), 2);
    let first: String = parser
        .screen()
        .line_cells(0)
        .unwrap()
        .iter()
        .map(vt100::Cell::contents)
        .collect();
    assert_eq!(first, "0123456789abcdefghij");
    assert_eq!(screen_rows(&parser), ["line3", "line4"]);
    let reflow = parser.screen_mut().take_reflow().unwrap();
    // Before: the two rows of the first line, then line2 .. line4.
    assert_eq!(reflow.lines, [Some(0), Some(0), Some(1), Some(2), Some(3)]);
    assert_eq!(reflow.dropped, 0);
    assert!(parser.screen_mut().take_reflow().is_none());
}

#[test]
fn a_wide_character_that_would_not_fit_goes_to_the_next_row() {
    let mut parser = reflowing(3, 20, 10);
    parser.process("abc中".as_bytes());
    resize(&mut parser, 3, 4);
    let first: String = parser
        .screen()
        .line_cells(0)
        .unwrap()
        .iter()
        .map(vt100::Cell::contents)
        .collect();
    assert_eq!(first, "abc");
    assert_eq!(screen_rows(&parser), ["中", "", ""]);
    assert_eq!(parser.screen().cursor_position(), (0, 2));
}

#[test]
fn the_alternate_screen_keeps_the_normal_one_until_it_goes() {
    let mut parser = reflowing(3, 20, 10);
    parser.process(b"abcdefghijklmnopqrstuvwxy\r\n\x1b[?1049h");
    resize(&mut parser, 3, 10);
    assert!(parser.screen_mut().take_reflow().is_none());
    parser.process(b"\x1b[?1049l");
    assert_eq!(screen_rows(&parser), ["klmnopqrst", "uvwxy", ""]);
    assert!(parser.screen_mut().take_reflow().is_some());
}

#[test]
fn a_history_too_long_for_its_capacity_drops_its_oldest_lines() {
    let mut parser = reflowing(2, 20, 1);
    parser.process(b"abcdefghijklmnopqrst\r\nx");
    resize(&mut parser, 2, 5);
    let reflow = parser.screen_mut().take_reflow().unwrap();
    assert_eq!(parser.screen().scrollback_len(), 1);
    // Five rows, two on the screen, one kept of the three for the history.
    assert_eq!(reflow.dropped, 2);
    assert_eq!(reflow.lines, [None, Some(2)]);
    assert_eq!(screen_rows(&parser), ["pqrst", "x"]);
}

#[test]
fn without_reflow_a_resize_cuts_rows_as_before() {
    let mut parser = helpers::new(3, 20, 0);
    parser.process(b"abcdefghijklmnopqrstuvwxy");
    resize(&mut parser, 3, 10);
    assert_eq!(screen_rows(&parser), ["abcdefghij", "uvwxy", ""]);
    assert!(parser.screen_mut().take_reflow().is_none());
}
