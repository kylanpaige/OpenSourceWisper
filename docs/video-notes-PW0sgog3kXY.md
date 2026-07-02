# Video notes — "STOP Using Claude Code Without This Fable 5 Agentic OS"

- **URL:** https://youtu.be/PW0sgog3kXY
- **Channel:** Chase AI (skool.com/chase-ai · chaseai.io)
- **Runtime:** ~20 min
- **Retrieved:** 2026-07-02, via scene-by-scene video analysis (YouTube's caption endpoints are
  bot-gated from datacenter IPs, so the spoken audio below is a near-verbatim machine
  reconstruction per scene rather than the official caption track; timestamps in the
  scene log drift slightly from the chapter markers in the description).

## Official description (excerpt)

> Your custom Claude Code Agentic OS can be yours in just three simple steps.
> In this video I breakdown how to setup the architecture, memory, and observability
> pieces of your Claude Code Agentic OS, giving you the ability to push your Claude Code
> efficiency to the limit.

Chapters: `0:00 Intro · 2:28 JARVIS walkthrough · 7:43 Architecture · 15:54 Skills + Customizing · 20:07 More Resources`

## The architecture shown in the video

The presenter demos **"Jarvis"**, his Claude Fable 5–built agentic OS layered on top of
Claude Code (built *with* Fable 5, but runnable on any model, largely local):

```
you speak
  └─► faster-whisper (local STT)
        └─► routing layer
              ├─ regex trigger words ("rundown", "run …") → direct skill dispatch
              ├─ Haiku 4.5 (cheap/fast API) → intent routing
              └─ or a fully local model via Ollama → intent routing
                    └─► headless Claude Code ("invisible" claude -p, pulls API credits)
                          └─► Obsidian vault (reports, memory, backlinks)
                                └─► summary → Kokoro (local TTS) → spoken reply
```

A **HUD dashboard** ("V.A.U.L.T.") sits on top:

- central animated orb + live audio meter (voice front-and-center, all local → snappy vs. ElevenLabs)
- **skills as buttons** — click to run any Claude Code skill without the terminal
  (queued → running → progress bar → report pop-up); the stated value: non-technical
  teammates/clients can run skills
- **Morning Report / Inbox Brief** — not hard-coded scripts; generated from reports in the
  Obsidian vault, with pop-ups linking to sources and "Open in Obsidian"
- **Schedule** — daily agenda linked to Google Calendar
- **Vitals** — follower/subscriber counts and other personal metrics
- **Directives** — top three things to work on today, dynamically generated
- **Document trail** — recently touched vault documents

Key quotes on the philosophy:

> "We've taken Claude Code and we've added another layer on top of it … things like visual
> metrics, things like skills that are turned into buttons. This isn't a productivity
> theater thing."

> "Its backbone is still this robust, completely customizable Claude Code skill
> architecture that takes everything you do in your day-to-day — your manual workflows,
> your daily tasks — and turns those into skills and automations."

> "We have this headless version of Claude Code. It's like opening up Claude Code but it's
> invisible … For personal-assistant type things you aren't going to run through two
> hundred dollars worth of credits."

> "You sit down with Claude Code, break your daily tasks out, and turn them into skills."
> (skills shown grouped into **Research**, **Content**, **Sales**)

## Scene-by-scene spoken audio (near-verbatim)

| # | Time | Spoken audio |
|---|------|--------------|
| 1 | 0:00–0:38 | "Hey Jarvis, give me the rundown for today." — "Good morning. You're at about 466,000 followers across all platforms, up about 3,000 this week. The latest video's pulling about 4,000 views a day, 17,000 so far. Big story in AI today: US government directive forced Anthropic to suspend public access to its top Mythos class. Biggest thing on today's board: cut and ship the Jarvis HUD reveal video. Want me to run the daily inbox audit or do you have anything else in mind?" |
| 2 | 0:38–0:58 | "So what exactly are we looking at here? Well, this is Jarvis, our Claude Fable 5 OS. Now, when we say Fable 5, I mean this was built using Fable 5, but it does not require us to have Fable 5 to actually run this. In fact, a lot of what you're seeing here is actually completely local and it can run on essentially any model you want." |
| 3 | 0:58–1:26 | "Now, if you have seen my previous Agentic OS videos, then you kind of know the deal here. We've taken Claude Code and we've added another layer on top of it, which gives us some things that you just can't get inside the terminal: things like visual metrics, things like skills that are turned into buttons. This isn't a productivity theater thing; this is something that gives us a true boost if we're operating in a bunch of different domains." |
| 4 | 1:26–1:30 | "And Jarvis is just the next evolution of this Agentic OS model." |
| 5 | 1:30–1:44 | "Its backbone is still this robust, completely customizable Claude Code skill architecture that takes everything you do in your day-to-day — your manual workflows, your daily tasks — and turns those into skills and automations." |
| 6 | 1:44–1:53 | "And it's on top of that bedrock that we build this. And in today's video, I'm going to show you how it works, where the true value lies, and how you can create something like this for yourself." |
| 7 | 1:53–1:59 | "And I think there's a lot of things you can take from this project, especially the local voice model dynamic we have going on." |
| 8 | 1:59–2:02 | "Before we dive into all that, a quick word from today's sponsor: me." |
| 9 | 2:02–2:28 | "So inside of Chase AI Plus, you not only have access to my exact setup you've seen in today's video, you also get the Claude Code Masterclass, which is the number one way to go from zero to AI dev. I update this every single week. We're currently running some deals on the membership, so if you want to take a look, just check out the pinned comment." |
| 10 | 2:28–2:42 | "So let's begin with a quick lay of the land with Jarvis so you can understand what it is you're actually looking at here. After we do that, we'll take a look under the hood, see how this is actually working so you understand how to customize it and how to build it yourself." |
| 11 | 2:42–3:34 | "So front and center, we have the whole voice aspect. Again, completely local, which allows it to be relatively fast and snappy compared to routing this all through something like ElevenLabs. And in the beginning, you heard Jarvis give me a whole spiel. That is not a hard-coded script. When I ask Jarvis, 'Hey, give me the rundown,' it takes a look at various reports automatically generated inside my Obsidian vault and determines what's actually important. You'll remember there were various pop-ups showing reports or links." |
| 12 | 3:34–3:37 | "Here's everything you need to know." |
| 13 | 3:37–3:41 | "And mentioned stuff about Anthropic…" |
| 14 | 3:41–3:45 | "It brings up the source article for that." |
| 15 | 3:45–4:00 | "It also talked about more things related to AI news. That all came from the Morning Report. So if I click this, you see this full write-up. This write-up lives inside Obsidian." |
| 16 | 4:00–4:04 | "And I can also click 'Open in Obsidian' and it brings up the original report." |
| 17 | 4:04–4:08 | "I can click on the different links. There's a whole connection here." |
| 18 | 4:08–4:27 | "You also remember it asked, 'Hey, do you want me to do that inbox triage for you?' Well, that is a skill. And those skills and automations are represented over here on the right. This allows me to instantly run any Claude Code skill with a click." |
| 19 | 4:27–4:49 | "Same exact idea. The value add here is more for if you're using this with a non-technical team or client and they want to be able to run skills without opening up the terminal." |
| 20 | 4:49–5:17 | "So let's say I did want to get a full inbox brief. If I just click 'Inbox Brief' over here on the top right, you can see it mentions it's queued right away. It mentions 'Running' and then we see a new pop-up and a progress bar showing that it's working." |
| 21 | 5:17–5:28 | "Inbox brief done. Eight threads, four sponsor pitches led by Out Skills, Paid Offer, Plus Minor Max, and Libernova. One fresh agency lead from Tameside and three filt…" |
| 22 | 5:28–5:35 | "So it gave me the quick verbal rundown and then I can see the actual report inside Obsidian." |
| 23 | 5:35–5:42 | "So these pop-ups are useful, relevant, and link us to things we actually care about." |
| 24 | 5:42–5:53 | "Down below that, we have the schedule. This is my daily schedule linked to my Google Calendar." |
| 25 | 5:53–5:57 | "If I click it, it brings up my calendar." |
| 26 | 5:57–7:22 | "We have a little audio section so you can see it moving. Over on the left, I show things like my subscriber counts. Down here are directives: the top three things I should be working on today, totally dynamic. And then I have a document trail. This is the user interface. This is the visual side of Jarvis." |
| 27 | 7:22–7:43 | "Now let's talk about the actual nuts and bolts, what's actually going on under the hood here because that's what actually matters. Let's be honest. It needs a proper backbone." |
| 28 | 7:43–8:24 | "And that's what we're looking at here. Let's walk through what happens when you talk to Jarvis. So here you are, and let's say you tell Jarvis, 'Give me an update on today's Morning Brief'. That audio goes to faster-whisper." |
| 29 | 8:24–10:05 | "Faster-whisper is going to take what you spoke and transcribe it. Now we need to figure out what to do with it. Regex picks up specific trigger words like 'rundown' and automatically routes it to take a look at the reports." |
| 30 | 10:05–11:55 | "But most of the time, what you're telling the AI needs a bit of intelligence to figure out where to route it. That's where we bring in Haiku because it's cheap and fast. We're simply routing. Option three is to have this be a completely local model like Ollama on your computer. Again, we're just routing here." |
| 31 | 11:55–12:40 | "So zooming out, we've given the request. Haiku's going to say, 'Okay, let's take a look at Obsidian'. Does this exist already? If not, it's then going to tell Claude Code to create the report. We have this headless version of Claude Code." |
| 32 | 12:40–13:02 | "It's like opening up Claude Code but it's invisible. It's going to pull from those API credits. Which can be a problem at huge scale, which is why you want to do a lot of these things with Sonnet." |
| 33 | 13:02–13:16 | "But I would argue it's not really a problem. The purpose of this is to act as a personal assistant, not to build Facebook. For personal-assistant type things, you aren't going to run through two hundred dollars worth of credits." |
| 34 | 13:16–15:05 | "Back to our example. Claude Code creates the report, it gets uploaded to Obsidian, and then it generates a summary. That goes to Kokoro, which turns it into words. Think of it as a mini ElevenLabs on our computer. That's what you heard today with Jarvis. This skill architecture is the backbone. You sit down with Claude Code, break your daily tasks out, and turn them into skills." (skills shown grouped: Research, Content, Sales) |
| 35 | 15:05–end | "So that's the whole system in a nutshell. I really like it mainly because of the customization. You can get pretty creative. There's nothing stopping you from bringing in Slack agents and that sort of thing. If you want my exact setup, link in the pinned comment. See you guys." |

## What we take from it for this repo

OpenSourceWisper's mission (fully local dictation) is the *voice front-end* of exactly this
stack. The agentic OS built in this repo ("Wisp OS") adopts the video's blueprint:

1. **Skill architecture as bedrock** — daily workflows live in `.claude/skills/`, shared by
   Claude Code (terminal) and the HUD (buttons).
2. **Routing layer** — regex trigger words first; then an optional LLM router
   (headless Claude / Anthropic API / Ollama); deterministic keyword fallback so the OS
   works with zero credentials.
3. **Headless Claude Code** — `claude -p` as the invisible execution engine, with a
   simulate mode when the CLI/credentials are absent.
4. **Markdown vault memory** — Obsidian-compatible `vault/` with reports, directives,
   inbox, and a document trail; every skill writes its results there.
5. **HUD observability** — a local dashboard: orb, vitals, directives, schedule,
   skills-as-buttons with queued/running/done states and report pop-ups.
6. **Local voice** — STT via faster-whisper (or this repo's `wisper` engine once the audio
   pipeline lands), TTS via Kokoro with OS-level fallbacks — all optional adapters.
