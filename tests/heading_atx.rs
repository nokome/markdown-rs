extern crate micromark;
use micromark::micromark;
#[test]
fn heading_atx() {
    assert_eq!(
        micromark("# foo"),
        "<h1>foo</h1>",
        "should support a heading w/ rank 1"
    );

    assert_eq!(
        micromark("## foo"),
        "<h2>foo</h2>",
        "should support a heading w/ rank 2"
    );

    assert_eq!(
        micromark("### foo"),
        "<h3>foo</h3>",
        "should support a heading w/ rank 3"
    );

    assert_eq!(
        micromark("#### foo"),
        "<h4>foo</h4>",
        "should support a heading w/ rank 4"
    );

    assert_eq!(
        micromark("##### foo"),
        "<h5>foo</h5>",
        "should support a heading w/ rank 5"
    );

    assert_eq!(
        micromark("###### foo"),
        "<h6>foo</h6>",
        "should support a heading w/ rank 6"
    );

    assert_eq!(
        micromark("####### foo"),
        "<p>####### foo</p>",
        "should not support a heading w/ rank 7"
    );

    assert_eq!(
        micromark("#5 bolt"),
        "<p>#5 bolt</p>",
        "should not support a heading for a number sign not followed by whitespace (1)"
    );

    assert_eq!(
        micromark("#hashtag"),
        "<p>#hashtag</p>",
        "should not support a heading for a number sign not followed by whitespace (2)"
    );

    assert_eq!(
        micromark("\\## foo"),
        "<p>## foo</p>",
        "should not support a heading for an escaped number sign"
    );

    // To do: attention.
    // assert_eq!(
    //     micromark("# foo *bar* \\*baz\\*"),
    //     "<h1>foo <em>bar</em> *baz*</h1>",
    //     "should support text content in headings"
    // );

    assert_eq!(
        micromark("#                  foo                     "),
        "<h1>foo</h1>",
        "should support arbitrary initial and final whitespace"
    );

    assert_eq!(
        micromark(" ### foo"),
        "<h3>foo</h3>",
        "should support an initial space"
    );

    assert_eq!(
        micromark("  ## foo"),
        "<h2>foo</h2>",
        "should support two initial spaces"
    );

    assert_eq!(
        micromark("   # foo"),
        "<h1>foo</h1>",
        "should support three initial spaces"
    );

    assert_eq!(
        micromark("    # foo"),
        "<pre><code># foo\n</code></pre>",
        "should not support four initial spaces"
    );

    // To do: trimming paragraphs.
    // assert_eq!(
    //     micromark("foo\n    # bar"),
    //     "<p>foo\n# bar</p>",
    //     "should not support four initial spaces when interrupting"
    // );

    assert_eq!(
        micromark("## foo ##"),
        "<h2>foo</h2>",
        "should support a closing sequence (1)"
    );

    assert_eq!(
        micromark("  ###   bar    ###"),
        "<h3>bar</h3>",
        "should support a closing sequence (2)"
    );

    assert_eq!(
        micromark("# foo ##################################"),
        "<h1>foo</h1>",
        "should support a closing sequence w/ an arbitrary number of number signs (1)"
    );

    assert_eq!(
        micromark("##### foo ##"),
        "<h5>foo</h5>",
        "should support a closing sequence w/ an arbitrary number of number signs (2)"
    );

    assert_eq!(
        micromark("### foo ###     "),
        "<h3>foo</h3>",
        "should support trailing whitespace after a closing sequence"
    );

    assert_eq!(
        micromark("### foo ### b"),
        "<h3>foo ### b</h3>",
        "should not support other content after a closing sequence"
    );

    assert_eq!(
        micromark("# foo#"),
        "<h1>foo#</h1>",
        "should not support a closing sequence w/o whitespace before it"
    );

    assert_eq!(
        micromark("### foo \\###"),
        "<h3>foo ###</h3>",
        "should not support an “escaped” closing sequence (1)"
    );

    assert_eq!(
        micromark("## foo #\\##"),
        "<h2>foo ###</h2>",
        "should not support an “escaped” closing sequence (2)"
    );

    assert_eq!(
        micromark("# foo \\#"),
        "<h1>foo #</h1>",
        "should not support an “escaped” closing sequence (3)"
    );

    assert_eq!(
        micromark("****\n## foo\n****"),
        "<hr />\n<h2>foo</h2>\n<hr />",
        "should support atx headings when not surrounded by blank lines"
    );

    assert_eq!(
        micromark("Foo bar\n# baz\nBar foo"),
        "<p>Foo bar</p>\n<h1>baz</h1>\n<p>Bar foo</p>",
        "should support atx headings interrupting paragraphs"
    );

    assert_eq!(
        micromark("## \n#\n### ###"),
        "<h2></h2>\n<h1></h1>\n<h3></h3>",
        "should support empty atx headings"
    );

    // To do: block quote.
    // assert_eq!(
    //     micromark("> #\na"),
    //     "<blockquote>\n<h1></h1>\n</blockquote>\n<p>a</p>",
    //     "should not support lazyness (1)"
    // );

    // assert_eq!(
    //     micromark("> a\n#"),
    //     "<blockquote>\n<p>a</p>\n</blockquote>\n<h1></h1>",
    //     "should not support lazyness (2)"
    // );

    // Extensions:
    // assert_eq!(
    //   micromark("# a", {extensions: [{disable: {null: ["headingAtx"]}}]}),
    //   "<p># a</p>",
    //   "should support turning off heading (atx)"
    // );
}
