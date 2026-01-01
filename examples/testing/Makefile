.PHONY: test_all

test_all:
	cargo run -q --example test_line_iter  -- < a.txt | diff -q expected_just_a -
	cargo run -q --example test_read_line  -- < a.txt | diff -q expected_just_a -
	cargo run -q --example test_read_until -- < a.txt | diff -q expected_just_a -
	cargo run -q --example test_line_iter  -- a.txt b.txt c.txt d.txt e.txt - g.txt h.txt i.txt j.txt k.txt l.txt < f.txt | diff -q expected_all -
	cargo run -q --example test_read_line  -- a.txt b.txt c.txt d.txt e.txt - g.txt h.txt i.txt j.txt k.txt l.txt < f.txt | diff -q expected_all -
	cargo run -q --example test_read_until -- a.txt b.txt c.txt d.txt e.txt - g.txt h.txt i.txt j.txt k.txt l.txt < f.txt | diff -q expected_all -
	cargo run -q --example test_reader     -- < a.txt | diff -q a.txt -
	cargo run -q --example test_reader     -- a.txt b.txt c.txt d.txt e.txt - g.txt h.txt i.txt j.txt k.txt l.txt < f.txt | diff -q expected_concat -
