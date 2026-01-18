/// Event Listener for themes change
function onChangeThemes(event) {
    window.removeEventListener(event.type, onChangeThemes);

    var themeToggleDarkIcon = document.getElementById('theme_toggle_dark_icon_target');
    var themeToggleLightIcon = document.getElementById('theme_toggle_light_icon_target');

    // Change the icons inside the button based on previous settings
    var is_theme_dark = localStorage.getItem('color-theme') === 'dark';
    var is_no_theme_in_local_storage = !('color-theme' in localStorage);
    var is_window_dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    var is_dark_icon_hidden = themeToggleDarkIcon.classList.contains('hidden');
    // var is_light_icon_hidden = themeToggleLightIcon.classList.contains('hidden');

    // check which icon should be enabled
    // if ((is_theme_dark || (is_no_theme_in_local_storage && is_window_dark)) && is_light_icon_hidden) {
    //     console.log("Dark Theme Toggled");
    //     themeToggleLightIcon.classList.toggle('hidden');
    // }
    if ((!is_theme_dark || (is_no_theme_in_local_storage && !is_window_dark)) && is_dark_icon_hidden) {
        themeToggleDarkIcon.classList.toggle('hidden');
    }

    // Add click event listener
    document.getElementById('theme_toggle_target').addEventListener('click', onChangeThemesClick);
}

/// Event Listener for themes change click
function onChangeThemesClick() {
    var themeToggleDarkIcon = document.getElementById('theme_toggle_dark_icon_target');
    var themeToggleLightIcon = document.getElementById('theme_toggle_light_icon_target');

    // toggle icons inside button
    themeToggleDarkIcon.classList.toggle('hidden');
    themeToggleLightIcon.classList.toggle('hidden');

    // if set via local storage previously
    if (localStorage.getItem('color-theme')) {
        if (localStorage.getItem('color-theme') === 'light') {
            document.documentElement.classList.add('dark');
            localStorage.setItem('color-theme', 'dark');
        } else {
            document.documentElement.classList.remove('dark');
            localStorage.setItem('color-theme', 'light');
        }

    // if NOT set via local storage previously
    } else {
        if (document.documentElement.classList.contains('dark')) {
            document.documentElement.classList.remove('dark');
            localStorage.setItem('color-theme', 'light');
        } else {
            document.documentElement.classList.add('dark');
            localStorage.setItem('color-theme', 'dark');
        }
    }
}

// Start event listeners
window.addEventListener("DOMContentLoaded", onChangeThemes);
window.addEventListener("htmx:afterSettle", onChangeThemes);
