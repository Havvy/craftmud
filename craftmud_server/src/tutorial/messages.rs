//! This module contains all of the strings for the tutorial.
// Instead of defining them everywhere, having them all in one easy to read
// location makes the process of working on the content easier than finding
// them on every dang state.

pub const INTRO_LEN: usize = 2;

pub const PREAMBLE: &str = "Starting tutorial. Welcome to MUDs!\r\n\r\n\
MUDs (Multi-user dungeons) are a kind of MMO where you interface with\r\n\
the world primarily with text. This tutorial will teach you the basics of\r\n\
playing in most MUDs. It will not teach you about how to do things that vary\r\n\
from MUD to MUD such as combat, leveling, and crafting.\r\n\r\n\
You've already performed the basic way of interacting with the MUD: sending\r\n\
a message when prompted. In this tutorial, you can use the command `next`\r\n\
to get the next page of the tutorial or a description of what to do to\r\n\
continue in the tutorial. You can also use `logout` to end the tutorial\r\n\
and go back to the login screen. You can also use `quitout` to quit out\r\n\
of the MUD entirely.\r\n\r\n\
Use `next` now to continue.\r\n";

pub const INTRO: [&str; INTRO_LEN] = [
"Good. now, in a MUD, you play a character who exists in a place. These are\r\n\
connected by exits. When you enter a new place, the MUD will show you a\r\n\
description of the place along with exits and other things of interest such as\r\n
the items on the ground or creatures to interact with.\r\n\r\n\
While this information is sent to you automatically, it's useful to check\r\n\
this information later. Such as when the description has scrolled off-screen.\r\n\
To do this `look` around.",

"You generally exist as a creature with an\r\n\
inventory located in a room. Rooms can have items to take, other players,\r\n",
];

//- Realm descriptions.

pub const ROOM_DESCS: [&'static str; 2] = [
    "A generic room",
    "A less generic room. You're in a hallway with windows. The windows let in\r\n
    sunlight."
];