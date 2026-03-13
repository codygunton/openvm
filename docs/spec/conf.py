# conf.py — Sphinx configuration for OpenVM STARK Specification
#
# Build HTML:  cd docs/spec && make html
# Build PDF:   cd docs/spec && make latexpdf
# Serve:       cd docs/spec && make livehtml

import os
import pickle
import sys
from unittest.mock import MagicMock

# -- Path setup ---------------------------------------------------------------
# Add executable-spec to sys.path so autodoc can import Python modules.
sys.path.insert(0, os.path.abspath("../../executable-spec"))
# Add docs/spec to sys.path so _ext can be imported.
sys.path.insert(0, os.path.abspath("."))

# -- Mock galois and FFI for autodoc/viewcode ---------------------------------
# The executable-spec uses galois (Galois field arithmetic) and poseidon2_ffi
# (Rust FFI for Poseidon2 hashing) which aren't available in the docs build.
for _mod in ["galois", "galois._fields", "galois._fields._gf",
             "galois._fields._array", "galois._domains", "galois._polys",
             "poseidon2_ffi"]:
    sys.modules[_mod] = MagicMock()

# galois field types are pickled in ff4_cache.pkl; patch pickle.load to
# return a mock instead of failing on the missing galois internals.
_orig_pickle_load = pickle.load
def _safe_pickle_load(f, **kwargs):
    try:
        return _orig_pickle_load(f, **kwargs)
    except Exception:
        return MagicMock()
pickle.load = _safe_pickle_load

# -- Project information ------------------------------------------------------
project = "OpenVM STARK Specification"
author = "Derived from the Python Executable Specification"
release = ""  # intentionally blank — suppresses "Release X" in PDF header

# -- General configuration ----------------------------------------------------
extensions = [
    "myst_parser",
    "sphinx.ext.autodoc",
    "sphinx.ext.viewcode",
    "autoapi.extension",
    "sphinx_copybutton",
    "sphinxcontrib.mermaid",
]

# MyST extensions for math + structured content
myst_enable_extensions = [
    "dollarmath",
    "amsmath",
    "deflist",
    "colon_fence",
]

# Numbered figures / tables / code blocks for cross-refs
numfig = True

# Suppress warnings from AutoAPI-generated rst files (docstring formatting)
suppress_warnings = ["docutils"]

# Source file suffixes
source_suffix = {
    ".md": "markdown",
    ".rst": "restructuredtext",
}

# -- Autodoc ------------------------------------------------------------------
autodoc_member_order = "bysource"

# -- AutoAPI (browsable module tree) ------------------------------------------
_spec = os.path.abspath("../../executable-spec")
autoapi_dirs = [
    os.path.join(_spec, "primitives"),
    os.path.join(_spec, "protocol"),
    os.path.join(_spec, "constraints"),
    os.path.join(_spec, "witness"),
]
autoapi_root = "api"
autoapi_type = "python"
autoapi_options = [
    "members",
    "undoc-members",
    "show-inheritance",
    "show-module-summary",
]
autoapi_own_page_level = "module"
autoapi_member_order = "bysource"
autoapi_add_toctree_entry = True
autoapi_keep_files = True
autoapi_python_use_implicit_namespaces = True
autoapi_ignore = [
    "*/tests/*",
    "*/poseidon2_ffi/*",
    "*/poseidon2-ffi/*",
    "*/.venv/*",
]

# -- HTML theme ---------------------------------------------------------------
html_theme = "sphinx_book_theme"
html_static_path = ["_static"]
html_css_files = ["custom.css"]
html_title = "OpenVM STARK Spec"
html_theme_options = {
    "show_toc_level": 2,
}

# -- Custom math macros -------------------------------------------------------
# Shared by MathJax (HTML) and LaTeX (PDF).
_MACROS = {
    r"\F":        r"\mathbb{F}_p",
    r"\Fext":     r"\mathbb{F}_{p^4}",
    r"\ZH":       r"Z_H",
    r"\MT":       r"\mathsf{MT}",
    r"\Hash":     r"\mathsf{H}",
    r"\Poseidon": r"\mathsf{Poseidon2}",
    r"\Fold":     r"\mathsf{Fold}",
    r"\INTT":     r"\mathsf{INTT}",
    r"\NTT":      r"\mathsf{NTT}",
    r"\LDE":      r"\mathsf{LDE}",
    r"\longto":   r"\longrightarrow",
    r"\longfrom": r"\longleftarrow",
}

# MathJax 3 configuration
# Load textmacros so \text{} handles \_ as underscore (and other LaTeX text commands).
mathjax3_config = {
    "loader": {"load": ["[tex]/textmacros"]},
    "tex": {
        "packages": {"[+]": ["textmacros"]},
        "macros": {
            k.lstrip("\\"): v
            for k, v in _MACROS.items()
        },
    },
}


# -- Custom {src} role --------------------------------------------------------

def _src_role(_name, rawtext, text, lineno, inliner, options=None, content=None):
    """Inline role for linking to viewcode source lines.

    Supports three reference formats:
    1. Legacy line number:  {src}`protocol/stark.py:202`
    2. Symbol reference:    {src}`protocol.stark.verify_stark`
    3. Anchor reference:    {src}`protocol/stark.py#verify-constraints`

    All formats support custom display text: {src}`Custom Text <path#anchor>`
    """
    import posixpath
    from docutils import nodes

    env = inliner.document.settings.env
    docname = env.docname

    # Check for custom display text: {src}`Custom Text <path#anchor>`
    custom_text = None
    if '<' in text and text.endswith('>'):
        custom_text, path_part = text.rsplit('<', 1)
        custom_text = custom_text.strip()
        text = path_part.rstrip('>')

    if '#' in text:
        # Anchor format: protocol/stark.py#verify-constraints
        file_path, anchor_id = text.rsplit('#', 1)
        if not file_path.endswith('.py'):
            file_path += '.py'

        anchor_cache = getattr(env.config, 'anchor_cache', None)
        if anchor_cache is None:
            msg = f"Anchor cache not available for reference '{text}'"
            return [inliner.reporter.error(msg, line=lineno)], []

        if file_path not in anchor_cache:
            msg = f"File '{file_path}' not found in anchor cache for reference '{text}'"
            return [inliner.reporter.error(msg, line=lineno)], []

        if anchor_id not in anchor_cache[file_path]:
            available = ', '.join(sorted(anchor_cache[file_path].keys())[:5])
            msg = f"Anchor '{anchor_id}' not found in {file_path}. Available: {available}..."
            return [inliner.reporter.error(msg, line=lineno)], []

        line_num = anchor_cache[file_path][anchor_id]
        display = custom_text if custom_text else f"{file_path.rsplit('/', 1)[-1]}#{anchor_id}"

    elif '.' in text and ':' not in text:
        # Symbol format: protocol.stark.verify_stark
        anchor_cache = getattr(env.config, 'anchor_cache', None)
        if anchor_cache is None:
            msg = f"Anchor cache not available for symbol reference '{text}'"
            return [inliner.reporter.error(msg, line=lineno)], []

        parts = text.split('.')
        file_path = None
        line_num = None

        for i in range(len(parts) - 1, 0, -1):
            candidate_path = '/'.join(parts[:i]) + '.py'
            if candidate_path in anchor_cache:
                if text in anchor_cache[candidate_path]:
                    file_path = candidate_path
                    line_num = anchor_cache[candidate_path][text]
                    break

        if file_path is None or line_num is None:
            msg = f"Symbol '{text}' not found in anchor cache"
            return [inliner.reporter.error(msg, line=lineno)], []

        display = custom_text if custom_text else parts[-1]

    elif ':' in text:
        # Legacy format: protocol/stark.py:202
        file_path, line_str = text.rsplit(':', 1)
        try:
            line_num = int(line_str)
        except ValueError:
            msg = f"Invalid line number in reference '{text}'"
            return [inliner.reporter.error(msg, line=lineno)], []
        display = custom_text if custom_text else f"{file_path.rsplit('/', 1)[-1]}:{line_str}"

    else:
        msg = f"Invalid reference format '{text}'. Expected 'path:line', 'module.symbol', or 'path#anchor'"
        return [inliner.reporter.error(msg, line=lineno)], []

    # Generate viewcode link
    module_path = file_path.replace('.py', '')
    target = f"_modules/{module_path}"
    rel = posixpath.relpath(target, posixpath.dirname(docname))
    url = f"{rel}.html#L-{line_num}"

    node = nodes.reference(rawtext, "", refuri=url, **(options or {}))
    if custom_text:
        node += nodes.inline(display, display)
    else:
        node += nodes.literal(display, display)
    return [node], []


# -- Viewcode line anchors patch ----------------------------------------------

def _patch_viewcode_line_anchors(app):
    """Patch Sphinx's Pygments bridge so viewcode pages get per-line anchors."""
    from sphinx.highlighting import PygmentsBridge
    _orig_init = PygmentsBridge.__init__

    def _patched_init(self, *args, **kwargs):
        _orig_init(self, *args, **kwargs)
        if hasattr(self, 'formatter') and self.formatter is not None:
            self.formatter.lineanchors = 'L'
            self.formatter.anchorlinenos = True
        if hasattr(self, 'formatter_args'):
            self.formatter_args['lineanchors'] = 'L'
            self.formatter_args['anchorlinenos'] = True

    PygmentsBridge.__init__ = _patched_init


def _init_anchor_cache(app, config):
    """Initialize anchor cache on config-inited event."""
    from pathlib import Path

    try:
        from _ext.anchor_scanner import load_anchor_cache

        base_path = Path(__file__).parent.parent.parent / "executable-spec"
        cache_path = Path(__file__).parent / "_anchor_cache.pkl"

        anchor_cache = load_anchor_cache(cache_path, base_path)
        config.anchor_cache = anchor_cache

    except ImportError as e:
        config.anchor_cache = None
        print(f"Warning: anchor_scanner not available ({e})")
    except Exception as e:
        config.anchor_cache = None
        print(f"Warning: Failed to build anchor cache: {e}")


def setup(app):
    """Sphinx setup hook."""
    _patch_viewcode_line_anchors(app)
    app.add_role("src", _src_role)
    app.connect("config-inited", _init_anchor_cache)


# -- LaTeX (PDF) output -------------------------------------------------------
latex_toplevel_sectioning = "part"

_latex_preamble = "\n".join(
    rf"\newcommand{{{k}}}{{{v}}}" for k, v in _MACROS.items()
)

latex_elements = {
    "papersize": "a4paper",
    "pointsize": "11pt",
    "extraclassoptions": "oneside",
    "fontpkg": r"\usepackage{lmodern}",
    "fncychap": "",
    "preamble": _latex_preamble + r"""

% Fix fancyhdr headheight warning
\setlength{\headheight}{13.6pt}

% Tighter page margins
\geometry{margin=1in}

% Clean hyperlink colors
\hypersetup{
  colorlinks=true,
  linkcolor=blue!70!black,
  citecolor=blue!70!black,
  urlcolor=blue!70!black,
}

% Compact lists
\usepackage{enumitem}
\setlist{nosep,leftmargin=*}

% Remove "Release" from running header
\renewcommand{\releasename}{}
""",

    "maketitle": r"""
\begin{titlepage}
\centering
\vspace*{3cm}
{\Huge\bfseries OpenVM STARK Specification\par}
\vspace{1.5cm}
{\Large Derived from the Python Executable Specification\par}
\vspace{2cm}
{\large \today\par}
\vspace{3cm}
\begin{minipage}{0.85\textwidth}
\noindent This document specifies the STARK proving system used by OpenVM.
It specifies \emph{exactly} what the prover and verifier compute,
using mathematical notation linked to the Python executable specification.

\medskip
\noindent The specification comprises two parts:
\begin{enumerate}[nosep]
  \item \textbf{STARK Protocol} --- the FRI-STARK proving system
        over the BabyBear field ($p = 2^{31} - 2^{27} + 1$) with
        quartic extension $\mathbb{F}_{p^4}$.
  \item \textbf{Programs} --- concrete instantiations: Fibonacci (single-AIR)
        and multi-AIR (RV32IM) examples demonstrating the protocol in action.
\end{enumerate}
\end{minipage}
\vfill
\end{titlepage}
""",

    "tableofcontents": r"\tableofcontents\clearpage",
    "printindex": "",
    "sphinxsetup": "verbatimwithframe=false, verbatimwrapslines=true",
}

latex_documents = [
    ("index", "openvm-stark-spec.tex", project, author, "manual"),
]
