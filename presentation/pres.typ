#import "@preview/codly:1.3.0": *
#import "@preview/typslides:1.3.2": *
#import "@preview/cetz:0.4.2"
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import fletcher.shapes: house, hexagon

// Project configuration
#show: typslides.with(
  ratio: "16-9",
  theme: "bluey",
  font: "Fira Sans",
  font-size: 20pt,
  link-style: "color",
  show-progress: true,
)

#front-slide(
  title: "Swiss Army Esp",
  subtitle: [An esp32 multitool],
  authors: "Riccardo Segala, Ettore Beltrame",
  info: [#link("https://github.com/RiccardoSegala04/swiss-army-esp")],
)

#slide(title: "System Overview")[
  #align(center,
    diagram(
      spacing: 8pt,
      cell-size: (5mm, 5mm),
      edge-stroke: 2pt,
      edge-corner-radius: 5pt,
      mark-scale: 100%,


      node((-0.5,-4), fill: blue.lighten(60%), stroke: 1pt + blue.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Driver 1],
      node((0,-4), fill: blue.lighten(60%), stroke: 1pt + blue.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Driver 2],
      node((0.5,-4), fill: blue.lighten(60%), stroke: 1pt + blue.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Driver 3],
      edge((-0.5, -2),(-0.5, -4), "<->", stroke: blue),
      edge((+0.5, -2),(+0.5, -4), "<->", stroke: blue),
      edge((0, -2),(0, -4), "<->", stroke: blue),
      node((-0.5,-2), fill: green.lighten(60%), stroke: 1pt + green.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Service 1],
      node((0,-2), fill: green.lighten(60%), stroke: 1pt + green.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Service 2],
      node((0.5,-2), fill: green.lighten(60%), stroke: 1pt + green.darken(30%), corner-radius: 10pt, height: 20mm, width: 50mm)[Service 3],
      edge((-0.4, 0),(-0.4, -2), "<-", stroke: green),
      edge((-0.6, 0),(-0.6, -2), "->", stroke: red),
      edge((+0.6, 0),(+0.6, -2), "<-", stroke: green),
      edge((+0.4, 0),(+0.4, -2), "->", stroke: red),
      edge((+0.1, 0),(+0.1, -2), "<-", stroke: green),
      edge((-0.1, 0),(-0.1, -2), "->", stroke: red),
      node((0,0), fill: yellow.lighten(60%), stroke: 1pt + yellow.darken(30%), corner-radius: 10pt, height: 20mm, width: 200mm)[Router],
      edge((-0.4, 0),(-0.4, 2), "->", stroke: green),
      edge((-0.6, 0),(-0.6, 2), "<-", stroke: red),
      edge((+0.6, 0),(+0.6, 2), "->", stroke: green),
      edge((+0.4, 0),(+0.4, 2), "<-", stroke: red),
      node((-0.5,2), fill: red.lighten(60%), stroke: 1pt + red.darken(30%), corner-radius: 10pt, height: 20mm, width: 100mm)[UI],
      node((0.5,2), fill: red.lighten(60%), stroke: 1pt + red.darken(30%), corner-radius: 10pt, height: 20mm, width: 100mm)[CLI],

    )
  )
  #align(center, [  #text(fill: red)[Commands channels], #text(fill: green)[Events channels],  #text(fill: blue)[Function calls] ])
]


#slide(title: "How it works")[

  - One background *Task* per Service
  - Communication via async *Channels* (FIFO message queues)
  - Actor-like model:
    - Services receive _Commands_
    - Services emit _Events_
  - No shared mutable state

  #framed[
    Clear boundaries between Drivers, Services, Router and UI.

    Hardware-independent interfaces above,
    hardware-specific drivers below.
  ]

] #slide(title: "Why Rust?")[
    - #stress("Safety without overhead")
      - Memory safety and no data races
      - No GC, no runtime, `no_std` friendly

    - #stress("embedded-hal ecosystem")
      - Hardware abstraction via traits
      - Portable drivers, easy mocking & testing

    - #stress("Embassy async")
      - `async/await` on bare metal
      - Natural fit for our task + channel architecture
      - No blocking, no manual state machines
]

#slide(title: "Code examples")[
  This is the run function that loops in the infrared service's task
  ```rust
  pub async fn run(&mut self) {
      loop {
          let comm = self.commands_receiver.receive().await;
          match comm {
              InfraredCommand::Play(sig) => {
                  self.ir.transmit(&sig).await;
                  self.send_event(InfraredEvent::SignalPlayed).await;
              }
              InfraredCommand::Listen => {
                  let ev = self.ir.listen().await;
                  self.send_event(ev).await;
              }
          }
      }
  }
  ```
]

#slide(title: "Testing and future Improvements")[

  #columns(2, gutter: 8pt)[

    - #stress("Test early, even without hardware")  
      - Stub platform-specific functions  
      - Use host virtual display implementation to preview UI

    - #stress("JTAG")
        - Instant, real-time debugging
        - Powerful log-level filtering

    #colbreak()

    #align(center, image("screenshot.png", width: 100%))

  ]


  
]

#slide(title: "Testing and future Improvements")[
  - #stress("New peripherals can be added:")
    - CAN Interface
    - SD Card 
    - ...
  - #stress("Also current peripherals could be used to add functionalities:")
    - Wi-Fi packets recording
    - MQTT Support for home automation
    - ...

  The fact that the code is modular and follows a service-based approach makes adding new Views, Devices and Services easy

]
