<script>
	import C from './C.svelte';
	let n = 7;
	const pr = Promise.resolve(1);
	let items = [1, 2];
</script>

<C><p slot="beta">{#each items as n}<b>{n}</b>{/each}</p><p slot="alpha">{#await pr then n}<i>{n}</i>{/await}</p><span>{#each items as _, n}<u>{n}</u>{/each}</span></C>
