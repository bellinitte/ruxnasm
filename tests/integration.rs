use difference::{Changeset, Difference};
use ruxnasm::assemble;

generator::generate_tests!();

pub fn print_diff(expected: &str, actual: &str) {
    let Changeset { diffs, .. } = Changeset::new(expected, actual, "\n");

    let mut t = term::stdout().unwrap();

    for i in 0..diffs.len() {
        match &diffs[i] {
            Difference::Same(x) => {
                t.reset().unwrap();
                writeln!(t, " {}", x).unwrap();
            }
            Difference::Add(x) => {
                match &diffs[i - 1] {
                    Difference::Rem(y) => {
                        t.fg(term::color::GREEN).unwrap();
                        write!(t, "+").unwrap();
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match &c {
                                Difference::Same(z) => {
                                    t.fg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z).unwrap();
                                    write!(t, " ").unwrap();
                                }
                                Difference::Add(z) => {
                                    t.fg(term::color::BLACK).unwrap();
                                    t.bg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z).unwrap();
                                    t.reset().unwrap();
                                    write!(t, " ").unwrap();
                                }
                                _ => (),
                            }
                        }
                        writeln!(t, "").unwrap();
                    }
                    _ => {
                        t.fg(term::color::BRIGHT_GREEN).unwrap();
                        writeln!(t, "+{}", x).unwrap();
                    }
                };
            }
            Difference::Rem(x) => {
                t.fg(term::color::RED).unwrap();
                writeln!(t, "-{}", x).unwrap();
            }
        }
    }
    t.reset().unwrap();
    t.flush().unwrap();
}
