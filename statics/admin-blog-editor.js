(function () {
    "use strict";

    function escapeHtml(text) {
        var div = document.createElement("div");
        div.textContent = text;
        return div.innerHTML;
    }

    function renderBlogPreviewContent(container) {
        if (!container || !container.querySelectorAll) return;
        if (typeof hljs !== "undefined") {
            container.querySelectorAll("pre code").forEach(function (el) {
                try {
                    hljs.highlightElement(el);
                } catch (err) {
                    console.error("Highlight.js render failed:", err);
                }
            });
        }
        if (typeof temml !== "undefined") {
            try {
                temml.renderMathInElement(container, { fences: "$+" });
            } catch (err) {
                console.error("Temml render failed:", err);
            }
        }
        if (typeof pele !== "undefined") {
            container.querySelectorAll("pre.mermaid").forEach(function (el) {
                try {
                    pele.mount(el, el.textContent, { zoom: false });
                } catch (err) {
                    console.error("Pele render failed:", err);
                }
            });
        }
    }

    function initTagWidget(widget) {
        var available = (widget.dataset.availableTags || "")
            .split(",")
            .map(function (t) { return t.trim(); })
            .filter(function (t) { return t.length > 0; });
        var selected = (widget.dataset.selectedTags || "")
            .split(",")
            .map(function (t) { return t.trim(); })
            .filter(function (t) { return t.length > 0; });

        var searchInput = widget.querySelector(".tag-search-input");
        var dropdown = widget.querySelector(".tag-dropdown");
        var selectedList = widget.querySelector(".tag-selected-list");

        if (!searchInput || !dropdown || !selectedList) return;

        function syncHiddenInputs() {
            widget.querySelectorAll('input[name="blog_tag"]').forEach(function (el) {
                el.remove();
            });
            selected.forEach(function (tag) {
                var input = document.createElement("input");
                input.type = "hidden";
                input.name = "blog_tag";
                input.value = tag;
                widget.appendChild(input);
            });
        }

        function renderSelected() {
            selectedList.innerHTML = "";
            selected.forEach(function (tag) {
                var chip = document.createElement("span");
                chip.className = "tag-chip";
                chip.innerHTML = escapeHtml(tag) + ' <button type="button" aria-label="Remove tag">&times;</button>';
                chip.querySelector("button").addEventListener("click", function () {
                    var idx = selected.indexOf(tag);
                    if (idx !== -1) selected.splice(idx, 1);
                    renderSelected();
                    renderDropdown(searchInput.value);
                });
                selectedList.appendChild(chip);
            });
            syncHiddenInputs();
        }

        function renderDropdown(query) {
            dropdown.innerHTML = "";
            var q = query.toLowerCase().trim();
            var alreadySelected = new Set(selected);
            var matched = available.filter(function (t) {
                return !alreadySelected.has(t) && t.toLowerCase().includes(q);
            });
            var hasItems = false;

            matched.forEach(function (tag) {
                hasItems = true;
                var item = document.createElement("div");
                item.className = "tag-dropdown-item";
                item.textContent = tag;
                // Prevent the mousedown default action so the search input
                // keeps focus. Otherwise the input's focusout handler hides
                // the dropdown between mousedown and mouseup and the click
                // never lands on the item.
                item.addEventListener("mousedown", function (e) {
                    e.preventDefault();
                });
                item.addEventListener("click", function () {
                    selected.push(tag);
                    renderSelected();
                    searchInput.value = "";
                    renderDropdown("");
                    searchInput.focus();
                });
                dropdown.appendChild(item);
            });

            var exactExists = available.some(function (t) { return t.toLowerCase() === q; });
            if (q && !exactExists && !alreadySelected.has(q)) {
                hasItems = true;
                var createItem = document.createElement("div");
                createItem.className = "tag-dropdown-item tag-create-item";
                createItem.textContent = 'Create new tag "' + q + '"';
                createItem.addEventListener("mousedown", function (e) {
                    e.preventDefault();
                });
                createItem.addEventListener("click", function () {
                    selected.push(q);
                    renderSelected();
                    searchInput.value = "";
                    renderDropdown("");
                    searchInput.focus();
                });
                dropdown.appendChild(createItem);
            }

            dropdown.classList.toggle("hidden", !hasItems);
        }

        searchInput.addEventListener("input", function () {
            renderDropdown(searchInput.value);
        });
        searchInput.addEventListener("focus", function () {
            renderDropdown(searchInput.value);
        });
        widget.addEventListener("focusout", function (e) {
            if (!widget.contains(e.relatedTarget)) {
                dropdown.classList.add("hidden");
            }
        });

        renderSelected();
    }

    function initBlogEditor(form) {
        if (!form) return;
        if (form.dataset.blogEditorInitialized) return;
        form.dataset.blogEditorInitialized = "true";

        var tagWidget = form.querySelector(".tag-widget");
        var textarea = form.querySelector("#blog_body");
        var previewBtn = form.querySelector('[hx-post="/admin/blogs/preview"]');
        var previewTarget = form.querySelector("#blog_preview");
        var tabBtns = form.querySelectorAll(".tab-btn");
        var panes = form.querySelectorAll(".body-editor-pane");

        // autoRefresh marks programmatic eye-button clicks issued by the
        // debounced body-input refresh so they do not also switch tabs.
        var autoRefresh = false;
        // previewDirty marks that the body changed since the last preview
        // render, so switching to the preview tab re-fetches instead of
        // showing stale content.
        var previewDirty = false;
        var previewTimeout;

        function isDesktop() {
            return !window.matchMedia || window.matchMedia("(min-width: 768px)").matches;
        }

        function activateTab(tabName, opts) {
            tabBtns.forEach(function (btn) {
                btn.classList.toggle("active", btn.dataset.tab === tabName);
            });
            panes.forEach(function (pane) {
                pane.classList.toggle("active", pane.dataset.pane === tabName);
            });
            var stalePreview = tabName === "preview"
                && opts && opts.requestPreview
                && previewTarget && previewBtn
                && (previewTarget.textContent.trim() === "" || previewDirty);
            if (stalePreview) {
                previewDirty = false;
                previewBtn.click();
            }
        }

        tabBtns.forEach(function (btn) {
            btn.addEventListener("click", function () {
                activateTab(btn.dataset.tab, { requestPreview: true });
            });
        });

        if (previewBtn) {
            previewBtn.addEventListener("click", function () {
                if (autoRefresh) {
                    autoRefresh = false;
                    return;
                }
                previewDirty = false;
                activateTab("preview");
            });
        }

        if (textarea && previewBtn) {
            textarea.addEventListener("input", function () {
                previewDirty = true;
                clearTimeout(previewTimeout);
                previewTimeout = setTimeout(function () {
                    if (!isDesktop()) {
                        // On mobile the preview pane is hidden behind a tab;
                        // fetch on demand when the preview tab opens instead.
                        return;
                    }
                    autoRefresh = true;
                    // The in-flight request renders the body as typed so far;
                    // later edits re-mark the preview dirty.
                    previewDirty = false;
                    previewBtn.click();
                }, 700);
            });
        }

        if (tagWidget) initTagWidget(tagWidget);
    }

    window.initBlogEditor = initBlogEditor;
    window.renderBlogPreviewContent = renderBlogPreviewContent;
})();
