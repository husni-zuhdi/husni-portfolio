/// Event Listener for themes change
function onChangeThemes() {
    var themeIconsDark = document.querySelectorAll('.theme_icon_dark');
    var themeIconsLight = document.querySelectorAll('.theme_icon_light');
    var themeButtons = document.querySelectorAll('.theme_button');

    if (themeIconsDark.length === 0 || themeIconsLight.length === 0) return;

    // Change the icons inside each button based on previous settings
    var is_theme_dark = localStorage.getItem('color-theme') === 'dark';
    var is_no_theme_in_local_storage = !('color-theme' in localStorage);
    var is_window_dark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    var effective_theme_is_dark = is_theme_dark || (is_no_theme_in_local_storage && is_window_dark);

    themeIconsDark.forEach(function (icon) {
        icon.classList.toggle('hidden', !effective_theme_is_dark);
    });
    themeIconsLight.forEach(function (icon) {
        icon.classList.toggle('hidden', effective_theme_is_dark);
    });

    // Add click event listener to every theme button (remove first to avoid duplicates)
    themeButtons.forEach(function (btn) {
        btn.removeEventListener('click', onChangeThemesClick);
        btn.addEventListener('click', onChangeThemesClick);
    });
}

/// Event Listener for themes change click
function onChangeThemesClick() {
    var themeIconsDark = document.querySelectorAll('.theme_icon_dark');
    var themeIconsLight = document.querySelectorAll('.theme_icon_light');

    // toggle icons inside every theme button in sync
    themeIconsDark.forEach(function (icon) {
        icon.classList.toggle('hidden');
    });
    themeIconsLight.forEach(function (icon) {
        icon.classList.toggle('hidden');
    });

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
