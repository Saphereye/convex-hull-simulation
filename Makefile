.PHONY: docs

docs:
	rm -rf docs
	mkdir -p docs
	RUSTDOCFLAGS="--html-in-header katex-header.html" cargo doc --no-deps
	cp -r target/doc/* docs/
	touch docs/index.html
	echo '<script>\n\twindow.location.href = "convex_hull_simulation/index.html";\n</script>' >> docs/index.html
	cp Read_this_for_info.md docs/Read_this_for_info.md

compress:
	tar -czvf Adarsh_Das_DAA_Submission.tar.gz docs