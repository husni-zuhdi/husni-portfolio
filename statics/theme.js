function WinLoad() {
    var themeToggleDarkIcon = document.getElementById('theme_toggle_dark_icon_target');
    var themeToggleLightIcon = document.getElementById('theme_toggle_light_icon_target');

    // Change the icons inside the button based on previous settings
    var is_theme_dark = localStorage.getItem('color-theme') === 'dark';
    var is_theme_in_local_storage = 'color-theme' in localStorage;
    var is_window_dark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    if (is_theme_dark || (!is_theme_in_local_storage && is_window_dark)) {
        themeToggleLightIcon.classList.remove('hidden');
    } else {
        themeToggleDarkIcon.classList.remove('hidden');
    }

    var themeToggleBtn = document.getElementById('theme_toggle_target');

    themeToggleBtn.addEventListener('click', function() {

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

    });
}

window.onload = WinLoad;
