window.Tssp = window.Tssp || {};

(function (T) {
  "use strict";

  T.confirmAction = function confirmAction(message) {
    return window.confirm(message);
  };

  T.promptText = function promptText(message, defaultValue = "") {
    return window.prompt(message, defaultValue);
  };
})(window.Tssp);
