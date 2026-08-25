<script>
	function declared() {
		return 1;
	}

	const alias = declared;
	const arrow = () => 1;
	const scalar = 1;
</script>

<b>{alias}</b>
<i>{arrow}</i>
<u>{scalar}</u>
