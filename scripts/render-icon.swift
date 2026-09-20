// Regenerate the app icon with: swift scripts/render-icon.swift
import AppKit

let size = 1024
let image = NSImage(size: NSSize(width: size, height: size))
image.lockFocus()
let background = NSBezierPath(roundedRect: NSRect(x: 72, y: 72, width: 880, height: 880), xRadius: 196, yRadius: 196)
let gradient = NSGradient(starting: NSColor(srgbRed: 0.15, green: 0.43, blue: 0.35, alpha: 1), ending: NSColor(srgbRed: 0.24, green: 0.61, blue: 0.48, alpha: 1))!
gradient.draw(in: background, angle: 65)
NSColor(srgbRed: 0.89, green: 0.98, blue: 0.92, alpha: 1).setStroke()
let stem = NSBezierPath()
stem.lineWidth = 38
stem.lineCapStyle = .round
stem.lineJoinStyle = .round
stem.move(to: NSPoint(x: 512, y: 285))
stem.line(to: NSPoint(x: 512, y: 704))
stem.move(to: NSPoint(x: 512, y: 430))
stem.curve(to: NSPoint(x: 340, y: 590), controlPoint1: NSPoint(x: 340, y: 430), controlPoint2: NSPoint(x: 340, y: 472))
stem.move(to: NSPoint(x: 512, y: 520))
stem.curve(to: NSPoint(x: 686, y: 670), controlPoint1: NSPoint(x: 686, y: 520), controlPoint2: NSPoint(x: 686, y: 582))
stem.stroke()
NSColor(srgbRed: 0.91, green: 0.98, blue: 0.93, alpha: 1).setFill()
NSBezierPath(ovalIn: NSRect(x: 460, y: 230, width: 104, height: 104)).fill()
NSBezierPath(ovalIn: NSRect(x: 291, y: 563, width: 98, height: 98)).fill()
NSBezierPath(roundedRect: NSRect(x: 642, y: 638, width: 88, height: 88), xRadius: 18, yRadius: 18).fill()
let leaf = NSBezierPath()
leaf.move(to: NSPoint(x: 512, y: 680))
leaf.curve(to: NSPoint(x: 575, y: 807), controlPoint1: NSPoint(x: 480, y: 741), controlPoint2: NSPoint(x: 526, y: 797))
leaf.curve(to: NSPoint(x: 512, y: 680), controlPoint1: NSPoint(x: 594, y: 746), controlPoint2: NSPoint(x: 569, y: 695))
leaf.fill()
image.unlockFocus()
let output = URL(fileURLWithPath: "assets/USBloom.iconset", isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
for points in [16, 32, 128, 256, 512] {
    for scale in [1, 2] {
        let pixels = points * scale
        let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: pixels, pixelsHigh: pixels, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
        let context = NSGraphicsContext(bitmapImageRep: bitmap)!
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = context
        context.imageInterpolation = .high
        image.draw(in: NSRect(x: 0, y: 0, width: pixels, height: pixels))
        NSGraphicsContext.restoreGraphicsState()
        let suffix = scale == 2 ? "@2x" : ""
        try bitmap.representation(using: .png, properties: [:])!.write(to: output.appendingPathComponent("icon_\(points)x\(points)\(suffix).png"))
    }
}
