<script>
	let prop = $state({ n: 1 });
	function use(a) {
		return a;
	}
</script>

<button
	onclick={() => {
		let counter = $state(1);
		let raw = $state.raw(1);
		counter = 2;
		raw = 3;
		use(counter + raw);
	}}
>a</button>

<button
	onclick={() => {
		let local = 1;
		local = 2;
		prop.n = local;
	}}
>b</button>

{prop.n}
