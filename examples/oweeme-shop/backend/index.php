<?php
/**
 * Backend de referencia en PHP para el `load()` de este ejemplo — la
 * prueba concreta del criterio de salida de la Fase 15: "un backend
 * distinto (PHP en vez del original) usa Nexa sin tocar el core". A
 * `load()` (crates/nexa-loader) no le importa en qué lenguaje esté
 * escrito el backend: solo hace un GET real y espera JSON.
 *
 * Arrancar: php -S 127.0.0.1:8090 backend/index.php
 * Luego:    NEXA_API_URL="http://127.0.0.1:8090" nexa preview
 */

header("Content-Type: application/json");

$products = [
    "iphone-17" => [
        "name" => "iPhone 17",
        "description" => "El último de Apple.",
        "image" => "https://oweeme.example/img/iphone17.jpg",
        "price" => 999,
    ],
    "pixel-10" => [
        "name" => "Pixel 10",
        "description" => "Lo último de Google.",
        "image" => "https://oweeme.example/img/pixel10.jpg",
        "price" => 799,
    ],
];

$path = parse_url($_SERVER["REQUEST_URI"], PHP_URL_PATH);

// Fase 16: catálogo completo para la isla `productFilter` del home —
// el mismo `$products` de arriba, con el slug incluido en cada entrada.
if ($path === "/products") {
    $catalog = [];
    foreach ($products as $slug => $product) {
        $catalog[] = array_merge(["slug" => $slug], $product);
    }
    echo json_encode(["products" => $catalog]);
    exit;
}

if (preg_match('#^/products/([a-z0-9-]+)$#', $path, $matches)) {
    $slug = $matches[1];
    if (array_key_exists($slug, $products)) {
        echo json_encode($products[$slug]);
        exit;
    }
    http_response_code(404);
    echo json_encode(["error" => "producto no encontrado"]);
    exit;
}

http_response_code(404);
echo json_encode(["error" => "ruta no encontrada"]);
