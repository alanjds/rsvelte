<script>
	let title = $state('title');

	function local(value) {
		return value;
	}
</script>

<div title={local(title)}></div>
<div title="{local(title).toUpperCase()}"></div>
<div title={String(title)}></div>
<button onclick={globalThis.make()}>global handler</button>
<Widget prop={local(title)} />
<svelte:element this="div" title={local(title)}></svelte:element>
