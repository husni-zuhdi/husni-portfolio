(function () {
    "use strict";

    const script = document.currentScript;
    const form = script && script.closest(".blog-editor-form");
    if (!form) return;

    const tagWidget = form.querySelector(".tag-widget");
    const textarea = form.querySelector("#blog_body");
    const previewBtn = form.querySelector('[hx-post="/admin/blogs/preview"]');
    const previewTarget = form.querySelector("#blog_preview");
    const tabBtns = form.querySelectorAll(".tab-btn");
    const panes = form.querySelectorAll(".body-editor-pane");

    function activateTab(tabName) {
        tabBtns.forEach(function (btn) {
            btn.classList.toggle("active", btn.dataset.tab === tabName);
        });
        panes.forEach(function (pane) {
            pane.classList.toggle("active", pane.dataset.pane === tabName);
        });
    }

    tabBtns.forEach(function (btn) {
        btn.addEventListener("click", function () {
            activateTab(btn.dataset.tab);
        });
    });

    if (previewBtn) {
        previewBtn.addEventListener("click", function () {
            activateTab("preview");
        });
    }

    let previewTimeout;
    if (textarea && previewBtn) {
        textarea.addEventListener("input", function () {
            clearTimeout(previewTimeout);
            previewTimeout = setTimeout(function () {
                previewBtn.click();
            }, 700);
        });
    }

    if (previewTarget) {
        previewTarget.addEventListener("htmx:afterSwap", function () {
            renderPreviewContent(previewTarget);
        });
    }

    function renderPreviewContent(container) {
        if (typeof temml !== "undefined") {
            temml.renderMathInElement(container, { fences: "$+" });
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

    function escapeHtml(text) {
        var div = document.createElement("div");
        div.textContent = text;
        return div.innerHTML;
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

    if (tagWidget) initTagWidget(tagWidget);
})();
