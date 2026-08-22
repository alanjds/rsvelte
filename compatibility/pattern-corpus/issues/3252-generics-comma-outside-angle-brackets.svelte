<script lang="ts" generics="T extends { a: 1, b: 2 }, U extends [1, 2]">
	let { value, pair }: { value: T; pair: U } = $props();
</script>

<p>{value.a}{pair[0]}</p>
