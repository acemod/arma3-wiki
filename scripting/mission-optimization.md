---
title: "Mission Optimisation"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
description: "Performance checklist: GPU/CPU/network impact by AI, objects, scripts, network messages, and unscheduled code. Includes diagnostic tools and server commands."
---

# Mission Optimisation

## Before Anything

Before optimising anything, make sure you do not have any performance issue running the game itself:

- Open the editor, place a unit in the area you like and test your computer.
  - You can get current FPS in Arma 3 by going into **video options** or using `diag_fps`
  - **Steam** allows you to display FPS in a screen corner, in *Settings > In game > FPS Counter*
- Play your mission in singleplayer. If your mission runs fine, its network messages might very well be the issue.
- Usual bottlenecks:
  - Lower your graphical settings (resolution, textures). If you get way better performances, at least your **GPU** limits you.
  - If the game keeps having low FPS when running at 1024x768/low textures then your CPU is most likely the issue. Mission scripts may be performance-hogging too.

> **Note:** **View distance** (among other settings) impacts both GPU and CPU!
>
> **Note:** View distance can be separately set for each client independently from the server's value through scripting. This can make or break performance for the client.

## Mission Creation

- Be sure to create your scripts with the latest available commands and functions.
  - In Arma 3 use `remoteExec` / `remoteExecCall` and **ditch `BIS_fnc_MP` for good!**
  - In Arma 2 network communication is done using the Arma 2 Multiplayer Framework.
- Use the available frameworks and functions for each topic, unless you replace them by third-party ones:
  - Arma 3: Respawn
  - Arma 3: Revive
  - Arma 3: Task Framework
  - Arma 3: Animated Briefing
  - Arma 3: Animated Opening
  - Arma 3: Dynamic Groups
  - Arma 3: Key Frame Animation
  - And most importantly: Functions

## Performance Impact Table

> - **Red** means a heavy impact on performance
> - **Orange** means an average impact
> - **Green** means little to no performance impact.

### AI unit quantity

| CPU | GPU | Network |
|-----|-----|---------|
| Red | Green | Orange |

- Use `createAgent` (Agents) whenever possible
- The more units there are, the more network updates there will be - hence the impact on both CPU and network.
- If a client has a low-end machine, they should not lead a group of many AIs as these would then be locally computed.
- Arma 3: Dynamic Simulation allows you to freeze AI that are distant from players. Many distance settings should be set according to the mission.

> **Note:** If you own more than one average computer, you could consider Headless Client to offload AI from the server.

### Object quantity

| CPU | GPU | Network |
|-----|-----|---------|
| Orange | Orange | Orange |

- The less objects, the more FPS you will have.
- Lower the quantity of objects in the mission, such as:
  - AI units, agents
  - Vehicles, simple objects
  - Weapon attachments (their proxies are `attachedTo`)
  - Headgear, clothing, vests, backpacks
  - Head-mounted devices (NVGs, etc.)
- Lower view distance
- Lower terrain details
- Lower the Dynamic Simulation threshold
- Use Simple Objects
- Use **Garbage Collection** in order to automatically delete bodies and wreckages:
  - Arma 3's Eden-integrated Garbage Collection
  - Arma 2's Garbage Collector

> **Note:** Only the **on-screen** objects will strongly impact the GPU.

### General script mistakes

| CPU | GPU | Network |
|-----|-----|---------|
| Orange | Green | Green |

Having too many scripts running is a cause for severe performance issues and execution delays in singleplayer as well as multiplayer.

- You can track the number of running scripts using `diag_activeScripts`; you can also use:
  - `diag_activeSQFScripts`
  - `diag_activeSQSScripts`
  - `diag_activeMissionFSMs`
- A common bad practice is to `spawn` a function for each unit that needs it; the good practice would be to have a single script working with an array of units, array that is edited (unit added/removed) whenever needed:

```sqf
{ _x spawn _myCode; } forEach _units;    // bad
_units spawn _myCode;                      // good
```

- You can use `diag_activeSQFScripts` to ensure you are not clogging your CPU with multiple instances of the same script.
- **Not compiling your scripts:** running multiple times the same script with `execVM` will make the game read the file every single time; This is an issue, especially if you execute it frequently.

```sqf
while { sleep 5; alive player } do { player execVM "myScript.sqf"; };    // bad

private _myScript = compile preprocessFileLineNumbers "myScript.sqf";
while { sleep 5; alive player } do { player spawn _myScript; };           // better

while { sleep 5; alive player } do { player spawn TAG_fnc_myScript; };   // perfect! see Functions
```

### High-frequency scripts

| CPU | GPU | Network |
|-----|-----|---------|
| Red | Green | Green |

Checking a condition too often is usually a source of poor performance. Does your code execution need to be frame-perfect, or can you afford a delay of a few seconds?

- A `while`-loop checking without a minimum loop `sleep` time is usually a sign of bad conception.

```sqf
while { true }                     // bad if you do not know what you are doing
while { alive player }             // better
while { sleep 1; alive player }    // perfect
```

- By default, **Triggers** check their set condition **every 0.5 second**. If a large area is covered or condition code is too complex, this can become an issue; the triggers should then be converted to scripts if possible. Since Arma 3 v2.00 it is possible to change trigger intervals, either via trigger attributes in 3DEN, or via the scripting command `setTriggerInterval`.
- **Triggers** can be made **Server-Side only** to save clients' resources.

> **Note:** All of the mission-related calculation (objectives, completion distance, etc.) must be done **server-side**. Local effects should be calculated **client-side**.

### High-frequency network messages

| CPU | GPU | Network |
|-----|-----|---------|
| Orange | Green | Red |

- Use `publicVariable` wisely; for specific cases, consider `publicVariableServer` / `publicVariableClient`
- Creating units/vehicles globally implies a network synchronisation, keep them to a minimum / at one-point in the mission.
- Keep **global effect** commands to a minimum:
  - e.g. a `setPos` will synchronise the unit position to every client, a good practice is to use these commands punctually. If you need a frequent "set position" you may want to look at `attachTo` depending on your usage.
  - **Global** marker commands always send **all** the marker's information. If you are creating/updating many markers e.g. server-side, edit them with **local** commands, except the last one (that will be global) to send the whole marker's status over the network, once.
- Lower client's view distance in order to lessen its object position update requests

### Unscheduled code

| CPU | GPU | Network |
|-----|-----|---------|
| Red | Green | Green |

Unscheduled code can have a high impact on the framerate, as such code is not subject to the scheduler's management (as its name suggests) and will run without limitation. Cyclic unscheduled low-performance code can make the game unplayable, up to freezing it.

- As only time-critical scripts should run unscheduled, `spawn` any other code from unscheduled environment once you have the required results:

```sqf
/* ... */ // unscheduled code
[_var1, _var2] spawn TAG_fnc_MyFunction;
/* ... */ // other unscheduled code
```

## Performance Diagnostic Tools

### Server Commands

- `#monitor 5` - Shows performance information of the server. Interval 0 means to stop monitoring.
- `#monitords 5` - Shows performance information in the dedicated server console. Interval 0 means to stop monitoring. (since Arma 3 v1.64)

### Diagnostic Commands

- Diag. commands only available in Diagnostic Branch
- Diag. command available in all builds

