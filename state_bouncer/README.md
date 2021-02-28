# State Bouncer

State bouncer's primary job is to watch the `~/Documents/Hearts of Iron IV/states` folder for new files so that it can
move them to `~/Documents/Hearts of Iron IV/mod/<mod id>/states`, which will let you create new states in Nudge and
click reload without having to move the state files yourself.
Currently, state bouncer also moves the `state_names_l_english.yml` localization file into your mod folder.
It could probably also support strategic region and supply area files, but I don't know how those work yet.

In order to make state bouncer work, you need to create a file in the same directory as the executable named `mod_id`
with your mod ID in it.
