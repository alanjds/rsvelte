<script>
	// The rewrite of the props-object reads was decided per line, and this line
	// carries a `$.prop(` call — so its default-value thunk, which is a genuine
	// read, was skipped along with the call's own binding-position argument.
	export let fromDefault = $$props.other;

	// Two reads on one line: both are reads, and neither is the first argument.
	export let twoReads = $$props.a ?? $$props.b;

	// The control that names the scanning unit rather than the rule. This read is
	// the same expression in the same component; it differs only in sitting on a
	// line the skip list never matched, and it was already correct.
	const fromBody = $$props.other;
</script>

<div {...$$restProps}>{fromDefault}{twoReads}{fromBody}</div>
