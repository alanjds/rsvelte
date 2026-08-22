use rsvelte_formatter::{FormatOptions, format};

#[test]
fn probe() {
    for src in [
        "<ul><li>a<li>b</ul>\n",
        "<ul><li>a</li><li>b</li></ul>\n",
        "<ul><li>a<li>b</li></ul>\n",
        "<ul><li>a</ul>\n",
        "<table><tbody><tr><td>a<td>b</tr></tbody></table>\n",
        "<dl><dt>a<dd>b</dl>\n",
        "<ul><li>a<ul><li>x</ul><li>b</ul>\n",
        "<div><p>a<p>b</div>\n",
        "<ul>\n\t<li>a\n\t<li>b\n</ul>\n",
    ] {
        let out =
            format(src, &FormatOptions::default()).unwrap_or_else(|e| format!("<<ERR {e:?}>>"));
        println!("IN : {src:?}\nOUT: {out:?}\n");
    }
}
