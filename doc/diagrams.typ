#import "typst-canvas/canvas.typ": canvas

#let linked_list(doubly: false) = align(center, text(size: 9pt,
  canvas(length: 1.25cm, {
    import "typst-canvas/draw.typ": *

    for (index, type) in ("Start", "Node", "Node", "End").enumerate() {
      fill(none)
      circle((index * 2, 0), radius: 0.5)
      content(
        (index * 2, 0),
        raw(type),
      )

      if index != 0 {
        fill(black);

        if doubly {
          line(
            (index * 2 - 1.525, 0.15),
            (index * 2 - 0.525, 0.15),
            mark-end: ">",
          )
          content(
            (index * 2 - 1, 0.35),
            text(8pt, raw("next"))
          )
          line(
            (index * 2 - 0.49, -0.15),
            (index * 2 - 1.5, -0.15),
            mark-end: ">",
          )
          content(
            (index * 2 - 1, -0.35),
            text(8pt, raw("prev"))
          )
        } else {
          line(
            (index * 2 - 1.5, 0),
            (index * 2 - 0.55, 0),
            mark-end: ">",
          )
          content(
            (index * 2 - 1, 0.2),
            text(8pt, raw("next"))
          )
        }
      }
    }
  })
))

#linked_list()
#linked_list(doubly: true)
