<script>
	let s = $state('v');
</script>

<select>
	<option value={() => 1}>a</option>
	<option value={function () {}}>b</option>
	<option value={1 + 1}>c</option>
	<option value={-1}>d</option>
	<option value={typeof s}>e</option>
	<option value={void 0}>f</option>
	<option value={s}>g</option>
</select>
