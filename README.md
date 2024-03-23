# r6rs
A General Purpose Discord Bot for Tom Clancy's Rainbow Six Siege designed to improve upon and run in tandem with [r6econ](https://github.com/hiibolt/r6econ).

All sections can be placed behind a Discord User ID whitelist system, allowing selective distribution!

![image](https://github.com/hiibolt/r6rs/assets/91273156/7bb4d86a-5aea-4a01-82c3-71ff607ffcd1)

## Features
This bot has a robust paywall system allowing access to be granted on a per-user, per section basis. There is one global admin who can assign lower-permission moderators, capable of granting users access.

There are 5 planned sections for this bot:
- [x] **Economy Monitor**
  
  Easily allows for personal manipulation of the R6 Marketplace by providing a wide host of information not otherwise visible on the marketplace. You can view optimal sale prices, visualize market trends, calculate rate average price, and much more!
- [x] **OSINT/OPSEC Scan Utility**

  Allows the lookup of otherwise hidden linked accounts via Ubisoft Webservices. You can find player's XBL, PSN, Steam, Epic, Amazon, Discord, and much, much more. Using a custom-tuned Sherlock Project behind a proxy, you can easily also guess their other potential platforms based on information found on linked accounts.
- [ ] **AI Cheater Scan Utility**

  Based on a huge amount of banned vs. legitimate player data, predict the confidence of a player in two categories: 'Rage Cheater (very obviously cheating)' or 'Closet Cheater (trying to hide their cheating status)'. Under development, the model has not yet finished training, it is currently estimated at 1/5~ complete on PaperSpace training machines.
- [ ] **Player Ban Monitor**

  Allows you to 'report' a player and recieve a notfication when that player is actually sanctioned with a temporary or permanent cheating ban. Under development.
- [x] **Administrator Utility**

  Makes administrator's jobs much easier. Can distribute administrator privledges to moderators who then have access to a restricted admin panel for distributing keys/whitelists.

## Bot Commands
### Economy Monitor
- `r6 econ analyze <item name | item id>`
  
  Lists all available trend information in text format.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/0c57034d-fd4c-463b-a8f9-7311b73f57fc)
- `r6 econ graph <item name | item id>`
  
  Presents an all-time graph of market trends.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/0f17912d-18af-47a0-8d17-ef637a2ed342)
- `r6 econ profit <purchased at> <item name | item id>`
  
  Calculates break-even price and current profit if sold, given the price you purchased said item at.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/22e7e800-354e-4c67-947f-403be5992346)
- `r6 econ list <(optional) page #>`
  
  Lists the currently tracked items.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/6e21f7e6-f1f2-4837-8236-51474e01fe09)
- `r6 econ help`

  Default page displayed in the event a subcommand is invalid.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/97f50fc8-f16e-4d17-9b68-3db962eaac4c)

### OPSEC/OSINT Lookup Utility
- `r6 opsec <pc | xbox | psn> <account name>`

  Fetches all linked accounts and queries Sherlock with the found usernames, if valid.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/56e43c5a-f0c2-472f-ad6b-ceeff1b59074)

- `r6 opsec namefind <username1> <username2> ...`

  Queries Sherlock instead of Ubisoft Webservices, if valid.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/f76b036a-ffbd-4233-833a-6cb77b279949)

- `r6 opsec help`

  Default page displayed in the event a subcommand is invalid.

  ![image](https://github.com/hiibolt/r6rs/assets/91273156/ac8b97c2-3738-41e1-ab27-947397bdcd8e)
## Setup
While the source code is open source, I will not be providing setup instructions. If you are interested in setting this up yourself, feel free, but do not contact me with issues unless something is explictly broken. 

You may always purchase access to sections that interest you by contacting @hiibolt on Discord!

### Credits
- @CNDRD on Discord for his well-documented auth renewal system, much love <3
- @UBI_FRAX on Twitter for their help with the OSINT section, actual goat <33

