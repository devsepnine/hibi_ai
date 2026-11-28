import Cocoa
import CoreGraphics

let bundle = URL(fileURLWithPath: CommandLine.arguments[1]).standardizedFileURL
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
for w in windows {
    guard let pid = w[kCGWindowOwnerPID as String] as? Int32,
          let app = NSRunningApplication(processIdentifier: pid),
          app.executableURL?.standardizedFileURL == bundle,
          w[kCGWindowLayer as String] as? Int == 0,
          let bounds = w[kCGWindowBounds as String] as? [String: Any],
          let width = bounds["Width"] as? Double, width >= 500 else { continue }
    print(pid)
    break
}
