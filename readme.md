# Plex Autotagger

This project is the successor to my plexUtils project, with the goal
of unifying everything into a single executable with very few runtime
dependencies.

When ripping TV shows from a DVD for use in a media server, they will
often come out of order, or there will be too many or too few titles,
with no obvious way of sifting through and organizing them. This
program will allow you to easily extract subtitles from mkv files,
compare them with subtitles downloaded from the internet, and use that
data to organize the video files in their correct order.

My first priority is getting this working. This means blocking the
async loop, unwraps, expects, etc. This will all be cleaned up before
the official release, but I want something I can use as soon as possible,
so I'm willing to write some dirty code to get there, but that doesn't
mean it'll stay that way. I want this to be a viable tool that empowers
users to obtain their content in an ethical way by making the process
easier.


## Dependencies

Currently, the tool uses third-party software to perform certain tasks.
A list of dependencies and resoning follows.

* mkvtoolsnix
  * This package contains the mkvextract command, used to extract
		subtitles from video files for further processing.
