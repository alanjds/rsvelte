<script>
	let s = $state('v');
</script>

<div is="a{s}b">x</div>
<div is="a">y</div>
<div is={s}>z</div>
