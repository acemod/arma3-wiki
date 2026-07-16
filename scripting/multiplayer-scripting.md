---
title: "Multiplayer Scripting"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
description: "Localisation rules, machine targeting (dedicated/player-server/client/headless), network ID, remote execution, publicVariable, setVariable, JIP synchronisation, and local testing procedures."
---

# Multiplayer Scripting

## Notions and general knowledge

### The basics

| Concept | Description |
|---------|-------------|
| **Server** | Distributes data (positions, speeds, etc.) to clients. Can be dedicated or player-hosted. |
| **Client** | A connected machine. Can join mid-mission (**JIP**). |
| **Singleplayer** | Game acts as player-hosted server (`isServer` returns true). |

> **Warning:** Some MP commands (e.g. `netId`) do **not** work in SP. Some SP commands (e.g. `setAccTime`) do **not** work in MP.

### Locality

**LOCAL** = machine where the command/script/function is executed. Check with `local`.
**REMOTE** = non-local (on another machine). Check with `not local _obj`.

| Type | Meaning |
|------|---------|
| Local Argument | Processed on the executing machine |
| Global Argument | Any argument, even remote |
| Local Effect | Effect only on executing machine |
| Global Effect | All machines notified |
| Server Exec | Must run on server |

#### Different machines and how to target them

> **Note:**
> - For obvious reasons, a server cannot be JIP.
> - Use `didJIP` to determine if a client joined during the game.

| Scripting Commands | Dedicated Server | Player-Hosted Server | Player Client | Headless Client | Note |
|--------------------|:---:|:---:|:---:|:---:|------|
| `isDedicated` | Yes | No | No | No | Dedicated server |
| `hasInterface && isServer` | No | Yes | No | No | Player server |
| `hasInterface && not isServer` | No | No | Yes | No | Player client |
| `not hasInterface && not isDedicated` | No | No | No | Yes | Headless client |
| `isServer` | Yes | Yes | No | No | Any server |
| `hasInterface` | No | Yes | Yes | No | Any player |

**Miscellaneous:**

| Scripting Commands | Note |
|--------------------|------|
| `not isServer` | All server clients, including headless clients, but excluding player host |
| `not isDedicated` | Everything but a dedicated server |
| `not hasInterface` | Headless clients and dedicated server |

**Cross-game targeting:**

| Targeted Machine | OFP | ArmA | ArmA2 | Arma 3 |
|-----------------|-----|------|-------|--------|
| **Dedicated Server**<br><small>A server without a human player behind it</small> | `isServer && isNull player` | `isServer && isNull player` | `isDedicated` | `isDedicated` |
| **Player Server**<br><small>A server hosted by a player</small> | `isServer && not isNull player` | `isServer && not isNull player` | `isServer && not isDedicated` | `hasInterface && isServer` |
| **Server**<br><small>Any server, dedicated or hosted</small> | `isServer` | `isServer` | `isServer` | `isServer` |
| **Player Client**<br><small>A player connected since the lobby</small> | `not isServer && not isNull player` | `not isServer && not isNull player` | `not isServer && not isNull player` | `hasInterface && not isServer` |
| **JIP Player Client**<br><small>A player connected in the middle of a mission</small> | N/A | `not isServer && isNull player` | `not isServer && isNull player` | `hasInterface && didJIP` |
| **Headless Client**<br><small>A client without a human player behind it (used to offload server calculations)</small> | N/A | N/A | N/A | `not hasInterface && not isServer` |
| **JIP Headless Client**<br><small>Headless client connected during the ongoing mission</small> | N/A | N/A | N/A | `not hasInterface && not isServer && didJIP` |

> **Note:** Before OFP 1.99 and the backport of `isServer`, the server could be identified by placing a Game Logic in the editor (that would always remain local to the server), then using `local myLogic`.

- If you want to know if the current machine is a server (Dedicated Server or Player Server), use `isServer`.
- If you want to know if the current machine has a player (Player Server included), use `hasInterface`.

> **Note:** Since Arma 3, a server-only init file is available: **initServer.sqf** (see Event Scripts).
>
> **Note:** Since Arma 3, `BIS_fnc_getNetMode` can be used to have a string value representing the local machine status ("DedicatedServer", "Server", "HeadlessClient", "Client", "SinglePlayer"). This function doesn't make the JIP distinction and should be used along `didJIP`.

#### Locality rules

| Entity | Local to |
|--------|----------|
| `player` unit | The (human) player's machine |
| Dedicated server | No `player` (`isNull player` = true) |
| AI group | Player-leader's machine |
| Driven vehicle | Driver's machine |
| Terrain objects | Everywhere |
| Editor-placed objects/empty vehicles | Server |
| Editor-placed triggers | Every machine (unless "Server Only") |
| `createUnit` / `createVehicle` | Command issuer's machine |

**Locality changes when:** player-leader dies, AI `join`s another group, player enters empty vehicle, Team Switch, `selectPlayer`.

> **Note:** Use `isPlayer` to check if a unit is a player. Use `hasInterface` for any player (including JIP).

#### Code examples

| Code | Arguments | Effect | Description |
|------|-----------|--------|-------------|
| `remoteUnit setDamage 1;` | Global Argument | Global Effect | Any unit (local or remote) will die, and all the computers will know |
| `localUnit addMagazine "30Rnd_556x45_STANAG";` | Local Argument | Global Effect | Only a local unit can have its inventory edited with this command, but all the machines will be notified: if a player looks into the localUnit inventory, the added magazine will be there |
| `remoteUnit setFace "Miller";` | Global Argument | Local Effect | Any unit (local or remote) can get its face changed, but the new face will not be propagated through the network. Only the local machine's player will see the effect of the command. |

> **Note:** This command should ideally be **executed on every machine**, for example with: `[remoteUnit, "Miller"] remoteExec ["setFace", 0, true];`

### Network ID

Network IDs identify machines and network objects. They can be used to target proper machines with remote execution.

> **Warning:** A machine ID (`owner`/ownerID) is not to be confused with an object's network ID (`netId`).

#### Machine network ID

Every machine, including the server, has a **network ID**. A server always has an ID of **2**, every new client has the next number (the first client will be number **3**, second client number **4**, etc.)

- To get the current machine's ID, use `clientOwner`.
- To get the machine ID of an object's owner, use `owner` object (MP only).
- To get the machine ID of a group's owner, use `groupOwner` object (MP only).

> **Note:** **0** and **1** are special values:
> - **0** means **everyone** (including the server)
> - **1** means **current machine** but **is not implemented** and should **not** be used.

#### Object network ID

Every object synchronised through the network has a network ID, shortened to **netId**.

- To get an object's netId, use `netId` (MP only). SP & MP variant: `BIS_fnc_netId`
- To get an object from a netId, use `objectFromNetId` (MP only). SP & MP variant: `BIS_fnc_objectFromNetId`
- To get a group from a netId, use `groupFromNetId` (MP only). SP & MP variant: `BIS_fnc_groupFromNetId`

### Sending information across network

#### Remote Execution

Since Arma 3 v1.50, `remoteExec` and `remoteExecCall` efficient engine network commands are available:

```sqf
"Message to everyone, including JIP players!" remoteExec ["hint", 0, true];
```

Arma 3 alpha introduced `BIS_fnc_MP` (**obsolete** since Arma 3 v1.50, use the above version):

```sqf
// obsolete since Arma 3 v1.50, use remoteExec or remoteExecCall
["Message to everyone, including JIP players!", "hint", true, true] call BIS_fnc_MP;
```

Arma 2 introduced the Arma 2 Multiplayer Framework (not present in Arma 3):

```sqf
// having the Functions module placed in the editor
waitUntil { not isNil "BIS_MPF_InitDone"; };
[nil, nil, rHINT, "Message to everyone!"] call RE;
```

#### PublicVariable commands

PublicVariable commands allow to send **global** variables to specified machines.

- Network reception is guaranteed
- Short variable names should be used for network performance
- These commands shouldn't be used intensely for the same reason

| Command | Effect | JIP synchronised |
|---------|--------|:---:|
| `publicVariable` | Set/update a variable value from the local machine to all the other machines (including server) | Yes |
| `publicVariableServer` | Set/update a variable value **from a client to the server** | No |
| `publicVariableClient` | Set/update a variable value **from the local machine** (not necessarily the server) **to a specific client** | No |

##### How it works

The server has the variable `ABC_Score` to broadcast:

```sqf
ABC_Score = 5;                              // sets the value
publicVariable "ABC_Score"                  // publishes the variable (do not forget the quotes!)
```

This sends the variable name and value to the clients.

- If the variable is not yet declared on their machine, it will declare it as well.
- If the variable already exists, **the value is overridden** by the server's value.
- If the variable changes again on the server, it has to be manually `publicVariable`'d once again!
- A JIP player will synchronise **server's** `publicVariable`'d variables **before** executing **init.sqf**. See Initialisation Order for more information.
- A JIP player will **not** synchronise `publicVariableClient`-received variables after he disconnects/reconnects. Only server-known public variables sent with `publicVariable` will be synchronised.

#### setVariable command

Available since Arma 2, the **public** version of the `setVariable` command (alternative syntax) allows you to store a variable into an object and spread this variable across the network.

> **Note:** In Arma 3 it is possible to broadcast the `nil` value, thus deleting the variable from the target.

### Player connection events

The following commands will execute given code when a player is connecting or disconnecting. This code will run for players connecting in the lobby **and** for JIP players!

- "PlayerConnected" mission Event Handler
- "PlayerDisconnected" mission Event Handler

> **Note:** Before Arma 3 v1.58, use `BIS_fnc_addStackedEventHandler` for `onPlayerConnected`/`onPlayerDisconnected`.

### Client state

A client state is the client's connection state. It can be obtained with `getClientState` and `getClientStateNumber` commands on both server and clients.

| getClientStateNumber | getClientState | Description |
|---------------------|----------------|-------------|
| 0 | "NONE" | No client (or singleplayer) |
| 1 | "CREATED" | Client is created |
| 2 | "CONNECTED" | Client is connected to server, message formats are registered |
| 3 | "LOGGED IN" | Identity is created |
| 4 | "MISSION SELECTED" | Mission is selected |
| 5 | "MISSION ASKED" | Server was asked to send / not send mission |
| 6 | "ROLE ASSIGNED" | Role was assigned (and confirmed) |
| 7 | "MISSION RECEIVED" | Mission received |
| 8 | "GAME LOADED" | Island loaded, vehicles received |
| 9 | "BRIEFING SHOWN" | Briefing was displayed |
| 10 | "BRIEFING READ" | Ready to play mission |
| 11 | "GAME FINISHED" | Game was finished |
| 12 | "DEBRIEFING READ" | Debriefing read, ready to continue with next mission |

### Join In Progress (JIP)

**Join In Progress** (JIP) is the ability for a player to connect while a game is running, unlike in OFP where one had to wait for the game to be over. It was introduced in OFPE on Xbox and ArmA on PC.

A player that joined in progress will be referred to as **JIP player**.

> **Note:** To disable JIP on a server:
> - The server admin can disable playable slots on the role selection screen
> - The mission designer can disable AI within mission config. However, JIP remains available if respawn is enabled in the mission.

#### JIP Synchronisation

A JIP player will have a lot of information synchronised (`publicVariable`'d and `setVariable` server variables for example) by the game **before** accessing the mission itself:

| Information | Arma 1 | ArmA2 | Arma 3 |
|-------------|:---:|:---:|:---:|
| date/time | Yes | Yes | Yes |
| date/time + `setDate`/`skipTime` | No | No | Yes |
| weather (`overcast`, `fog`) | No | Yes | Yes |
| weather + `setOvercast`/`setFog`/`setRain`/`setLightnings` | No | No | Yes |
| time passed since mission start (`time` command) | ? | ? | Yes |
| `publicVariable`-sent variables (Number, String, Structured Text, Array, Code) | Yes | Yes | Yes |
| `publicVariable`-sent variables (`nil`) | No | No | Yes |
| `setVariable`-assigned variables (when alternative syntax's **public** parameter is set to true) | N/A | Yes | Yes |
| `remoteExec`- and `remoteExecCall`-executed code if JIP prerequisites are met | N/A | N/A | Yes |

> **Warning:** All the **initialisation fields** code get executed again for **each and every JIP connecting player**. This means that code with global effect (such as `setDamage`) **WILL** be re-executed (depending on locality)!

#### Related Commands

- `remoteExec`
- `remoteExecCall`
- `didJIP`
- `didJIPOwner`
- `exportJIPMessages`
- `isRemoteExecutedJIP`

## Structuring Multiplayer code

### Responsibilities

- The **server** initialises and works with **variables**.
- A connecting or connected **player-client** gets the mission data from the server.
- Once an objective is reached, **the server tells the players** via `remoteExec`/`remoteExecCall`, `publicVariable` or `setVariable`.
- Players can get their client-side UI code triggered by the server.

See earlier chapter (Different machines and how to target them) to know how to target specific machines.

#### Server

The **Server** is the authority: all decisions come from it, all values on the server are **true**. Heavy logic runs here. It can be assisted by Headless Clients.

- Gather variables (from clients via `publicVariableServer`), broadcast decisions via `publicVariable`/`remoteExec`.
- Avoid `player` — servers can be dedicated. Use `allPlayers`, `playableUnits`, `switchableUnits` instead.

#### Client

**No mission-vital code** on clients. Any client can disconnect at any moment.

Clients handle local UI (Post Process Effects, Dialogs). Check for player-side with:

```sqf
if (hasInterface) then { /* player-side code */ };
```

> **Note:** A server **can** be a player. `isServer`/`else` is incorrect for player checks.

### Code writing

The usual four steps of coding are the following:

1. **Think it well**
   - If you cannot figure out how to write your code, say what you want to do out loud or write it down in your language, then replace it by code little by little.
2. **Make it work**
   - Use the `-showScriptErrors` startup parameter to see your code errors.
3. **Make it readable**
   - Name your variables properly, as if you had to give it to someone that should get it without the need of explaining.
4. **Optimise then**
   - Use more performance-friendly commands, reduce your search radius and loop frequencies once everything works.

## Local Multiplayer Testing

To ensure all cases, your mission should ideally work properly:
- In Singleplayer
- In Player-hosted Multiplayer
- In a Dedicated Server-hosted game

The following chapters explain how to mimic multiple clients on one computer. Of course, nothing prevents you from inviting friends to play your mission and report bugs!

### Dedicated Server

To properly test MP scripting locality issues, it is recommended to run a **dedicated server** and connect with **two** clients. In order to do so:

1. In your server executable's `server.cfg`:

```cpp
loopback = true;            // force LAN-only server
kickDuplicate = 0;          // disable duplicate Arma kick

// needed in case of Headless Client test only
headlessClients[]    = { "127.0.0.1" };
localClient[]       = { "127.0.0.1" };
```

2. Run the game twice from Steam and connect each (with `-showScriptErrors` flag)
3. If you use one screen for the first client and the other screen for the second client, use `-noPause -window` launcher options. This will not pause render if the window doesn't have focus.

> **Note:** Be sure to disable BattlEye or it will close any additional instances.

4. Try your mission and check for script errors:
   - On client's screens and RPT files
   - In your server's RPT files

| Pros | Cons |
|------|------|
| Tests almost all "can go wrong" scenarios | Misses cases where `isDedicated` is used instead of `isServer` |
| Can test Headless Clients, persistent server, server restart, admin login/logout | "Big" setup |

### Player-Hosted Server

While this method covers less cases than the Dedicated Server Test method above, this method is faster:

1. Open the Arma 3 Launcher and disable BattlEye
2. Select the mods that the host will use (if any)
3. Launch the game using the Play / Play with Mods button
4. Select the mods that clients will use
5. Launch the game again using the Play / Play with Mods button
6. In the first game instance (server), host the mission either through local hosting or with Eden Editor MP mission type launch
7. In the second game instance (client), join the session using the MP tab LAN section and entering the mission hosted by the server

| Pros | Cons |
|------|------|
| Fast setup (with quick modlist support) | May miss locality issues (code that happens to run on the server because it runs on the server's **client** like e.g. `hideObjectGlobal`) |
| Helps find cases where `isDedicated` is used instead of `isServer` | No headless client testing (this can be solved by using a custom `server.cfg`) |
