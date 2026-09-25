---
title: "Code Positivity"
category: "scripting-tutorial"
source: "scriptingpageswiki.txt"
---

# Code Positivity

This page provides some insights into what it means to try and write code in a positive way, with the least negation as possible.

## Overview

Code negation is (usually) using `not` (in SQF/SQS) or `!` (in both SQF/SQS and Enforce Script); all in all, it is about searching for the opposite of what one wants. Examples:

| SQF | Enforce Script |
|-----|----------------|
| ```sqf
if (not alive player) then
{
    hint "you are dead";
}
else
{
    hint "you are alive";
};
``` | ```enforce
if (value < minValue)
{
    Print("Value is invalid");
}
else
{
    Print("Value is valid");
}
``` |

Research has shown that **negation** in general (not just in code but also in reading, teaching etc) makes processing sentences more difficult for the human brain, making its message less impactful (although not in all cases).

## Examples

| Do | Don't |
|----|-------|
| ```sqf
// this is called Early Return / Bouncer pattern
if (isNull _control1) exitWith {};
if (isNull _control2) exitWith {};
if (isNull _control3) exitWith {};

// do something
``` | ```sqf
if (not isNull _control1) then
{
    if (not isNull _control2) then
    {
        if (not isNull _control3) then
        {
            // finally do something

            // <- this is called Hadouken code style as per
            // the shape drawn by the brackets
        };
    };
};
``` |
| ```enforce
// whenever possible without rewriting everything
// or doubling the methods, of course
if (unit.IsAlive() && unit.IsInjured())
    DoHealThing();
``` | ```enforce
// alternatively
if (unit.IsDead())
    DoDeathThing();
else if (unit.IsPerfectHealth())
    DoPerfectThing();
else // injured
    DoHealThing();
``` | |
| ```enforce
// sometimes, only negative methods are available
if (!unit.IsPerfectHealth() && !unit.IsDead())
    DoHealThing();
``` | |

## Counter Examples

As most if not all rules of development, this rule is a rule of thumb that is **not** to be enforced at all cost.

| Don't | Do |
|-------|----|
| ```sqf
if (isNull _control) then
{
}
else
{
    systemChat "the control exists";
};

// or, even more convoluted and less performance-friendly
if (str _control find "NULL" == -1) then
{
    systemChat "the control exists";
};
``` | ```sqf
if (not isNull _control) then // neater
{
    systemChat "the control exists";
};
``` |
| ```enforce
if (m_aElements.Count() > 0)
    Print("There are elements!");
// note that in some cases, storing the count in a variable
// saves further calculations in below code
``` | ```enforce
if (!m_aElements.IsEmpty())
    Print("There are elements!");
``` |

