# r6rs
<img src="https://github.com/hiibolt/hiibolt/assets/91273156/9528b9af-4166-4b51-b3f8-084d75dccc3b" width="200"/>

### About
A General Purpose Discord Bot for Tom Clancy's Rainbow Six Siege designed to improve upon and run in tandem with [r6econ](https://github.com/hiibolt/r6econ), but containing multiple powerful OSINT tools. 

Includes many, many utilities for gathering open source intelligence via a variety of paid APIs in a succint, error-checked, and pretty looking package.

All sections can be placed behind a Discord User ID whitelist system, allowing selective distribution via a key system.

All commands are alternatively available as slash commands that better indicate what constitutes each argument, as well as whether a given argument is optional.

## Packaged Utilities 
- Sherlock
- BulkVS - CNAM Lookup
- Snusbase - Leak Queries
- Snusbase - Geolocation
- Snusbase - Hash Queries
- Rainbow Six Siege Social API
- Rainbow Six Siege Marketplace GraphQL API


## Commands

### Admin

` >>admin whitelist <section> <user id>`
- Adds a person to the authorized user list.

` >>admin dm <user id> <message>`
- DMs a message to a specific user.

` >>admin announce <sections> <message>`
- Announces a message to all whitelisted users.

` >>admin blacklist <section> <user id>`
- Removes a person from the authorized user list.

### OSINT

#### Hashing

` >>osint hash dehash <hash>`
- Dehashes a hash into pre-cracked passwords.

` >>osint hash rehash <password>`
- Rehashes a password into pre-hashed hashes.

#### Queries

` >>osint query username <username>`
- Queries for leaks based on a username.

` >>osint query name <name>`
- Queries for leaks based on a name.

` >>osint query password <password>`
- Queries for leaks based on a password.

` >>osint query ip <ip>`
- Queries for leaks based on a last IP.

` >>osint query hash <hash>`
- Queries for leaks based on a hash.

` >>osint query email <email>`
- Queries for leaks based on an email.

#### Other

` >>osint sherlock <username>`
- Cross-references sites with a given username.

` >>osint geolocate <ip>`
- Geolocates an IP.

` >>osint phone <phone number>`
- Perform a Caller ID lookup on a phone number.

### R6

#### OPSEC

` >>r6 opsec psn <username>`
- Looks up a Ubisoft account based on their registered PSN username.

` >>r6 opsec applications <username>`
- Looks up a Ubisoft account based on their username (PC only).

` >>r6 opsec pc <username>`
- Looks up a Ubisoft account based on their registered PC username.

` >>r6 opsec xbox <username>`
- Looks up a Ubisoft account based on their registered Xbox username.

#### Economy

` >>r6 econ analyze <item name | item id>`
- Creates a detailed data sheet on an item.

` >>r6 econ transfer`
` >>r6 econ transfer <ubisoft email> <ubisoft password>`
- Finds the items with the least sellers either globally or on the account with the provided login.

` >>r6 econ list`
` >>r6 econ list <page #>`
- Lists all available skins.

` >>r6 econ profit <$ bought for> <item name | item id>`
- Calculates the amount you would make if you sold your item right now.

` >>r6 econ graph <item name | item id>`
- Graphs the all-time history of an item.


## Setup
While open source, I do *not* provide setup instructions. If such a tool interests you, join the [Discord](https://discord.gg/ENGqjywsbm) for details on purchasing access!