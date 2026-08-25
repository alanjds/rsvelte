<script>
	let maybe = $state();
</script>

<select>
	<option value={() => 1}>arrow</option>
	<option value={function () {}}>function</option>
	<option value={1 + maybe}>binary</option>
	<option value={-maybe}>unary</option>
	<option value={void 0}>undefined unary control</option>
	<option value={maybe}>unknown control</option>
</select>
