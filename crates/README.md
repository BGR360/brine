# Brine architecture

At the very highest level, the architecture of Brine looks like this:

```
              ┌──────────────────┐              
              │                  │              
          ┌──▶│  Brine Protocol  │◀──┐          
          │   │                  │   │          
          │   └──────────────────┘   │          
          │                          │          
          │                          │          
          ▼                          ▼          
┌──────────────────┐       ┌──────────────────┐ 
│                  │       │                  │ 
│    Client App    │       │     Backend      │ 
│                  │       │                  │ 
└┬─────────────────┘       └┬─────────────────┘ 
 │   .─────────────.        │   .─────────────. 
 ├─▶(   Rendering   )       ├─▶(  Networking   )
 │   `─────────────'        │   `─────────────' 
 │   .─────────────.        │   .─────────────. 
 ├─▶( Player Input  )       └─▶(  MC Protocol  )
 │   `─────────────'            `─────────────' 
 │   .─────────────.                            
 └─▶(    Assets     )                           
     `─────────────'                            
```

The project is based around a high-level abstraction of the Minecraft game
logic, referred to as the "Brine protocol". This protocol is defined by a set of
event types that are exchanged between the client application and the "backend",
which is the portion of the client that actually handles communicating with
the Minecraft server.

## Bevy at the core

This project is structured from the ground up as a Bevy application. Most crates
are structured as one or more Bevy plugins. Understanding how all the pieces fit
together and interact will require a basic understanding of the Bevy ECS and app
system. Start with the
[Bevy book](https://bevyengine.org/learn/book/introduction/).

## Crate details

```
              ┌───────┐                      
              │ brine │──────────────┐       
              └───────┘              │       
                  │                  │       
                  │                  ▼       
                  │           ┌─────────────┐
                  │           │ brine_voxel │
                  │           └─────────────┘
                  │                  │       
                  │                  │       
                  ▼                  ▼       
            ┌───────────┐     ┌─────────────┐
            │brine_proto│────▶│ brine_chunk │
            └───────────┘     └─────────────┘
                  ▲                  ▲       
┌───────────┐     │                  │       
│ brine_net │     │                  │       
└───────────┘     │                  │       
      ▲           │                  │       
      └───────────┤                  │       
                  │                  │       
       ┌─────────────────────┐       │       
       │ brine_proto_backend │───────┘       
       └─────────────────────┘               
```

### [`brine_chunk`](brine_chunk/)

Logic for decoding chunk data from Minecraft packets.

No Bevy dependencies.

### [`brine_data`](brine_data/)

Provides access to Minecraft data for any version.

No Bevy dependencies.

### [`brine_net`](brine_net/)

A library for implementing client-server protocols over TCP.

### [`brine_proto`](brine_proto/)

A high-level abstraction of the Minecraft game logic.

### [`brine_proto_backend`](brine_proto_backend/)

A backend implementation for Minecraft Java Edition. Currently powered by the
`steven_protocol` crate from the
[`stevenarella`](https://github.com/iceiix/stevenarella) project.

### [`brine_voxel`](brine_voxel/)

A library for rendering chunked voxel worlds.
