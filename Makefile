expand:
	@cargo expand -p test_suite --test config_loader

publish:
	# Correct order since configloader imports derive hehe
	@cargo publish -p configloader_derive
	@cargo publish -p configloader
