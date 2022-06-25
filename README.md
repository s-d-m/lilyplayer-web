Lilyplayer-web
=======

If you are interested in lilyplayer, you can see its code at https://github.com/s-d-m/lilyplayer .

If you want a live demo, in your browser, you can use it at https://lilyplayer-web.shuttleapp.rs/

It takes some time to load as it downloads about 50MiB at the start.

Once loaded, click, "input", then "select file". Suitable files for the application are available at
https://github.com/s-d-m/precompiled_music_sheets_for_lilyplayer . Simply download music sheets you like
from there.


Why this repository?
=====

I ported Lilyplayer to Webassembly make it usable directly in the browser[^1]. Initially, I planned to simply
host the html files on a repository here, to make the web app easily available at no cost (for me).  However,
the app uses threads, which is restricted to websites with cross site isolation only. Browsers (firefox,
chrome, ...) know if a website should run in cross site isolation mode or not based on http headers[^2] sent by
the web server when accessing the main html page. And the webserver powering github does not provide these
headers. Sadly, I couldn't find any hosting platform letting me host static files for free and whose webserver were setting
those headers. However, I came across shuttle.rs which is a platform letting me run a webserver which I can control.
Consequently, I made my toy web-server which only serves the files required to run lilyplayer and set those
magic http headers. Now lilyplayer is a 50MiB application, and embedding the data directly inside the webserver
didn't work due to a size limit of shuttle.rs. So instead, the files are hosted here on github, and the webserver
will download them first to send them after. Essentially, the webserver is a very simple proxy.

This repository contains the code of that webserver. The assets (i.e. html and javascript files) are available
on a different branch.


[^1]: Note that the native app provides a nicer user experience.
[^2]: The headers are: "Cross-Origin-Opener-Policy" and "Cross-Origin-Embedder-Policy". They must to set to
	  "same-origin" and "require-corp" respectively
