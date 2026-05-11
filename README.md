# "Broken" DMG on MacOS
The DMG isn't broken, it's complaining that the code isn't signed. I don't want to pay $99 a year to Apple to make a developer account so for now use the following work around
- Run `xattr -cr "/Applications/Twitch Follows.app"` into your terminal (replace path to app if it's different, but it should default to this)
- This command removes the "Quarantine" flag which MacOS applies to any unsigned software downloaded from the internet.

