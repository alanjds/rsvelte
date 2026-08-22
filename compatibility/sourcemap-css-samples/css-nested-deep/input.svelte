<div class="foo">
	<span class="bar">deep</span>
</div>

<style>
	.foo {
		color: red;

		& .bar {
			color: blue;

			&:hover {
				color: lime;
			}
		}
	}
</style>
