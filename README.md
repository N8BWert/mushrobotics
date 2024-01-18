# Mushrobotics

## Description

A few years ago, I worked with a team at a hackathon to make a simple prototype of a mushroom farming robot.  That project can be found [here](https://github.com/N8BWert/Pleurotus-Ostreatus-Automaton).  I have always loved the idea so this project is meant to go the extra step and create a fully functional mushroom farming robot (ideally one that is small enough I can keep it in my apartment).  However, I am not satisfied with just making a singular mushroom farming robot because that is truly boring.  Instead, I would like to create a network of mushroom farming robots in a sort of [OpenThread](https://openthread.io/) inspired IoT farming project.  Ultimately, I would like to branch out from farming mushrooms (possibly to farm radishes, something I've wanted to do for a very long time), but mushrooms have incredibly short growing cycles and growing oyster mushrooms is incredibly easy.

Also, I want to do this in Rust because I really think that it is the perfect language for robotics.  It combines the performance of C with a bit of the flexibility of higher level languages, which I just think is neat.

## Motivation

I have always loved robotic farming.  In fact, if I could do any job, my preferred job would be robotic farming.  Anyways, I was sitting at work and I thought to my self, "What am I doing here?"  I love what I do at work, but my passion will always be in robotic farming so I realized that I should actually do something about that.  So as a Biff like character I saw the sky and am going to do my best to pursue this project because I simply want to.

## Mushrobotics Home

In the OpenThread standard, there are multiple types of connected devices.  One of which is the thread router (and border router).  In my mind, the Mushrobotics Home will be run on a Raspberry Pi Zero (likely running Raspbian Lite or another light Unix-based operating system) that can serve as a connection from the internet to my mushroom robots.  I would like to be able to monitor the robots remotely from any internet enabled devices so hopefully I can use Rocket or Axum to run a web server off of this border router so I can access my mushroom robotics from wherever I am.

## Mushrobotics Add

I'm not currently happy with the name, but Mushrobotics Add is another device that can be added to the network of devices that can serve as OpenThread's Sleepy End Device (SED).  The idea of a SED is that the device can consume as little power as possible by only waking up at discrete intervals while the thread router buffers instructions to it.  I like this idea, so I would like to make a few additional mushroom farming robots that can serve as sleepy end devices, only waking up periodically to send information to the Mushrobotics Home device.  In addition, the Mushrobotics Add will likely be run with a Raspberry Pi Pico (my personal favorite microcontroller) and will, therefore, be an embedded program (probably using RTIC, my favorite RTOS) which will both present its own challenges and rewards.

#### Last Updated: January 18, 2024