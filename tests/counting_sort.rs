extern crate scanner_rust;

use std::fmt::Write;

use scanner_rust::Scanner;

const INPUT_DATA_PATH: &str = r"tests/data/input_1.txt";

#[test]
fn counting_sort() {
    let mut sc = Scanner::scan_path(INPUT_DATA_PATH).unwrap();

    let n = sc.next_usize().unwrap().unwrap();

    let mut a = Vec::with_capacity(n);
    let mut count = vec![0; 100];

    for i in 0..n {
        a.push(sc.next_usize().unwrap().unwrap());
        count[a[i]] += 1;
    }

    let mut s = String::new();

    for (i, &v) in count.iter().enumerate().take(100) {
        if v > 0 {
            for _ in 0..v {
                s.write_fmt(format_args!("{} ", i)).unwrap();
            }
        }
    }

    assert_eq!("1 1 3 3 6 8 9 9 10 12 13 16 16 18 20 21 21 22 23 24 25 25 25 27 27 30 30 32 32 32 33 33 33 34 39 39 40 40 41 42 43 44 44 46 46 48 50 53 56 56 57 59 60 61 63 65 67 67 68 69 69 69 70 70 73 73 74 75 75 76 78 78 79 79 80 81 81 82 83 83 84 85 86 86 87 87 89 89 89 90 90 91 92 94 95 96 98 98 99 99", s.trim());
}
