import Foundation

/// Relays the extension's native messages to the marks API. It decides nothing:
/// it checks the message shape and the route, forwards, and returns the API's reply.
final class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {
    private static let apiBase = "http://127.0.0.1:47821"
    private static let allowedRoutes = ["/v1/ping": "GET", "/v1/bookmarks": "POST"]
    private static let requestTimeout: TimeInterval = 30

    // Safari wraps the payload under this key, in both directions.
    private static let envelopeKey = "message"
    private static let methodKey = "method"
    private static let pathKey = "path"
    private static let tokenKey = "token"
    private static let bodyKey = "body"
    private static let statusKey = "status"
    private static let errorKey = "error"
    private static let badRequest = "bad_request"
    private static let unreachable = "unreachable"
    private static let authorizationHeader = "Authorization" // i18n-allow: format
    private static let bearerPrefix = "Bearer " // i18n-allow: format
    private static let contentTypeHeader = "Content-Type"
    private static let jsonContentType = "application/json"

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message = item?.userInfo?[Self.envelopeKey] as? [String: Any]
        Self.relay(message) { reply in
            let response = NSExtensionItem()
            response.userInfo = [Self.envelopeKey: reply]
            context.completeRequest(returningItems: [response], completionHandler: nil)
        }
    }

    private static func relay(_ message: [String: Any]?, completion: @escaping ([String: Any]) -> Void) {
        guard let request = makeRequest(message) else {
            return completion([errorKey: badRequest])
        }
        URLSession.shared.dataTask(with: request) { data, response, error in
            guard error == nil, let http = response as? HTTPURLResponse else {
                return completion([errorKey: unreachable])
            }
            // A non-JSON body (or a non-object one) becomes an empty object.
            let body = data.flatMap { try? JSONSerialization.jsonObject(with: $0) as? [String: Any] } ?? [:]
            completion([statusKey: http.statusCode, bodyKey: body])
        }.resume()
    }

    private static func makeRequest(_ message: [String: Any]?) -> URLRequest? {
        guard let message,
              let method = message[methodKey] as? String,
              let path = message[pathKey] as? String,
              let token = message[tokenKey] as? String,
              allowedRoutes[path] == method,
              let url = URL(string: apiBase + path)
        else { return nil }
        var request = URLRequest(url: url, timeoutInterval: requestTimeout)
        request.httpMethod = method
        request.setValue(bearerPrefix + token, forHTTPHeaderField: authorizationHeader)
        if let body = message[bodyKey] {
            // Unescaped slashes keep HTML at the size Chromium sends, so the 8 MB limit matches.
            guard JSONSerialization.isValidJSONObject(body),
                  let data = try? JSONSerialization.data(withJSONObject: body, options: .withoutEscapingSlashes)
            else { return nil }
            request.setValue(jsonContentType, forHTTPHeaderField: contentTypeHeader)
            request.httpBody = data
        }
        return request
    }
}
