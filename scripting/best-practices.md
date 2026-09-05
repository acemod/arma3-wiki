---
title: "Code Best Practices"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
description: "Prerequisites, code format, variable naming conventions, code structuring and organisation principles, with good/bad code examples."
---

# Code Best Practices

## Prerequisites

- An advanced text editor such as Visual Studio Code or Notepad++
- Some English knowledge, helped if needed by a translator tool
- Tutorials and examples (YouTube tutorials, community tutorials, example code)
- Search engine skills
- ARMA Discord server: [#arma3_scripting](https://discord.gg/arma3_scripting)

## Best Practices

### Code Format

- Whatever you do about your code's format, **be consistent**.
- Choose an indentation format **and stick to it**. There is not especially one indent better than another; the important point is **consistency**.
  - Common indentation styles:
    - [K&R style](https://en.wikipedia.org/wiki/Indentation_style#K&R_style) indenting
    - [Allman style](https://en.wikipedia.org/wiki/Indentation_style#Allman_style) indenting
  - Use empty space. Line return, spaces before and after brackets, if this improves readability, use it: space is free.
  - Indent with two/four spaces **or** one tab. Do not mix these.
  - **One-lining** (putting everything in one statement) memory improvement is most of the time not worth the headache it gives when trying to read it. Don't overuse it.
- `0 = myCommand` **was** "useful" only for editor fields that for no apparent reason refused commands returning a value. You do **not** need it in script files, and don't need it in editor fields anymore since Arma 3 v2.04.

### Variable Format

- Name your variables and functions properly:
  - While SQF **is** (non-noticeably) impacted by variable name length, this should not take precedence on the fact that code must be readable by a human being.
  - Your variable names must have a meaning: variables like `_u` instead of `_uniform` should not exist. `_i` is an accepted iteration variable name (e.g. in `for` loops).
  - It is recommended to use [camel-case](https://en.wikipedia.org/wiki/Camel_case) for your variables; camel-casing (namingLikeThis) your variables and functions makes the code naturally more readable, especially for long names.
- Prefix your public variables and `setVariable` with your tag in order to avoid any conflict about other mods, scripts or mission variables.
- Make your variables **private** thanks to the `private` or `params` keyword in order to avoid accidental upper-scope overwriting of variables of the same name.
- Defined constants must be UPPERCASE_WITH_UNDERSCORES (e.g. `#define SOME_CONST`).

### Code Structuration

- **DRY:** **D**on't **R**epeat **Y**ourself. If you write the same code block or the same logic many times, export this code as a function and use parameters with it.
  - If your code has too many levels, it is time to split and rethink it - maybe using `exitWith` (e.g. `if (a) then { if (b) then { if (c) then { /* etc */ }; }; };`)
  - Do **NOT** use macros as functions - these hinder readability. Use functions instead.
- Comments in your code must not explain **what** the code does, but **why** it is done this way (if needed). Your code organisation combined to your variable names must be enough to be read by a human.

### Code/Files Organisation

- Use CfgFunctions to declare the functions that will be called frequently.
  - One Functions directory, with sub-directories if needed.
- Use Event Scripts as needed.
- Do not put **any code** in a unit's init box **but eventually** local commands for this specific unit - **all** unit's init boxes are run client-side on client connection.

## Examples

### Good Practice Examples

| Bad Example | Good Example |
|-------------|--------------|
| `_unit = player;` | `private _unit = player;` |
| `private _uB = allUnits select { side _x == blufor };` | `private _bluforUnits = allUnits select { side _x == blufor };` |
| `private _plead = leader player;` | `private _playersLeader = leader player;` |
| `finalAssault = true; publicVariable "finalAssault";` | `PREFIX_finalAssault = true; publicVariable "PREFIX_finalAssault";` |
| `player setVariable ["MoneyInPocket", 250, true];` | `player setVariable ["PREFIX_MoneyInPocket", 250, true];` |
| ```sqf
#define KILL(UNIT) UNIT setDamage 1

// ...

{
    KILL(_x);
} forEach (units group player - [player]);
``` | ```sqf
private _killFnc = { _this setDamage 1; };

// ...

{
    _x call _killFnc;
} forEach (units group player - [player]);
``` |
| ```sqf
// player will need health at this stage of the mission
if (damage player > 0.25) then
{
    // if the player has no first aid kit
    if (not ("FirstAidKit" in items player))
    {
        // if player has room for first aid kit
        if (player canAdd "FirstAidKit") then
        {
            // add first aid kit to the player
            player addItem "FirstAidKit";
        }
        else
        {
            // set player's damage to 1/4
            player setDamage 0.25;
        };
    };
};
``` | ```sqf
// player will need health at this stage of the mission
if (
    damage player > 0.25 &&
    not ("FirstAidKit" in items player)) then
{
    if (player canAdd "FirstAidKit") then
    {
        player addItem "FirstAidKit";
    }
    else // let's help him anyway
    {
        player setDamage 0.25;
    };
};
``` |

### Flow Logic Examples

| Bad Example | Good Example |
|-------------|--------------|
| ```sqf
if (alive player && damage player >= 0.9) then {
    hint "very damaged";
};
if (alive player && damage player >= 0.5 && damage player < 0.9) then {
    hint "quite damaged";
};
if (alive player && damage player > 0 && damage player < 0.5) then {
    hint "slightly damaged";
};
if (alive player && damage player == 0) then {
    hint "pristine state";
};
if (not alive player) then {
    hint "dead";
};
``` | ```sqf
if (not alive player) exitWith { hint "dead"; };

private _playerDamage = damage player;
switch (true) do {
    case (_playerDamage >= 0.9): { hint "very damaged"; };
    case (_playerDamage >= 0.5): { hint "quite damaged"; };
    case (_playerDamage > 0)   : { hint "slightly damaged"; };
    default { hint "pristine"; };
};
``` |
| ```sqf
if (cond1) then
{
    if (cond2) then
    {
        if (cond3) then
        {
            // hadouken code
        }
        else { /* failure 3 */ };
    }
    else { /* failure 2 */ };
}
else { /* failure 1 */ };
``` | ```sqf
if (!cond1) exitWith { /* failure 1 */ };
if (!cond2) exitWith { /* failure 2 */ };
if (!cond3) exitWith { /* failure 3 */ };

// code
``` |

### Format Examples

| Bad Example | Good Example |
|-------------|--------------|
| ```sqf
if (not alive player) exitWith 
{ hint "dead"; };

private _playerDamage = damage player;
switch (true) do {
case (_playerDamage >= 0.9): {hint "very damaged";};
case (_playerDamage >= 0.5): {
    hint  "quite damaged";};

case (_playerDamage > 0): { hint "slightly damaged"; };
    default
    {

     hint "pristine";
    };
};
``` | ```sqf
if (not alive player) exitWith { hint "dead"; };

private _playerDamage = damage player;
switch (true) do {
    case (_playerDamage >= 0.9): { hint "very damaged"; };
    case (_playerDamage >= 0.5): { hint "quite damaged"; };
    case (_playerDamage > 0)   : { hint "slightly damaged"; };
    default { hint "pristine"; };
};
``` |

## Final Words

- Remember: **consistency** is the most important thing!
- The ARMA Discord server and its community can help: [ARMA Discord](https://discord.gg/arma) (channel `#arma3_scripting`)
- Learn from others' scripts but do not steal code and pretend it is yours — be a decent human being. Stealing code and its consequences:
  - It soils your reputation and devalues your actions once it is found out — and it **always** gets found out. DMCA's are filled on Steam every day.
  - It makes people in the community get less helpful and more reluctant in giving advice. It can also prevent them to **release** an interesting feature!
- Do not try to obfuscate your code: it is considered rude, especially since you learnt from others.
  - Obfuscated code only makes it **harder** to get, but does not make it **copy-protected**. If the engine can read it, it can be obtained.
  - Obfuscated code is also slower on parsing and, depending on the quality of code and obfuscation, on execution too.
- Do not hesitate to help newcomers even if you do not know the whole topic: both sides will learn from it!
- Most important of all, have fun!

