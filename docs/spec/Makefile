# Makefile for OpenVM STARK Specification
SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

.PHONY: help html latexpdf clean livehtml

help:
	@echo "  html      Build HTML documentation"
	@echo "  latexpdf  Build PDF documentation"
	@echo "  clean     Remove build artifacts"
	@echo "  livehtml  Serve with auto-rebuild on changes"

html:
	$(SPHINXBUILD) -b html $(SPHINXOPTS) $(SOURCEDIR) $(BUILDDIR)/html
	@echo "Build finished. Open $(BUILDDIR)/html/index.html"

latexpdf:
	$(SPHINXBUILD) -b latex $(SPHINXOPTS) $(SOURCEDIR) $(BUILDDIR)/latex
	cd $(BUILDDIR)/latex && make
	@echo "Build finished. PDF at $(BUILDDIR)/latex/openvm-stark-spec.pdf"

clean:
	rm -rf $(BUILDDIR) _anchor_cache.pkl api/

livehtml:
	sphinx-autobuild $(SPHINXOPTS) $(SOURCEDIR) $(BUILDDIR)/html --watch ../../executable-spec
