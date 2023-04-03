window.onload = function() {
  //<editor-fold desc="Changeable Configuration Block">

  // the following lines will be replaced by docker/configurator, when it runs in a docker-container
  window.ui = SwaggerUIBundle({
    // config where to fetch the swagger.json config
    url: "/docs/openapi/api.json",
    // config the default model load or example data load
    defaultModelRendering: "model",
    dom_id: '#swagger-ui',
    defaultModelsExpandDepth: 10,
    defaultModelExpandDepth: 10,
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
      SwaggerUIBundle.plugins.DownloadUrl
    ],
    layout: "StandaloneLayout"
  });

  //</editor-fold>
};
