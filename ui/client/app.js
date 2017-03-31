window.onload = function () {

  var but = document.getElementById('buttonsend');
  but.addEventListener('click', but_send, false);

  // var socket = new WebSocket('ws://echo.websocket.org');
  var socket = new WebSocket('ws://127.0.0.1:42575');
  socket.onopen = function (event) {
    console.log("conntection established ...")
    socket.send("hoitaus");
  };
  socket.onmessage = function (event) {
    var message = event.data;
    console.log(message)
    var param = JSON.parse(message)
    $("#slider").slider("value", param.value);
    console.log(param);
  };

  function but_send() {
    socket.send("hoitaus");
    console.log("clicked");
  }

  $(function () {
    $("#slider").slider({
      min: -60,
      max: 0,
      step: 0.5,
      slide: function (event, ui) {
        console.log("sending: " + ui.value);
        var param = {
          key: 2,
          value: ui.value
        };
        socket.send(JSON.stringify(param));
      }
    });
  });
};
