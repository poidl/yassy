window.onload = function () {

  var socket = new WebSocket('ws://127.0.0.1:55555');
  socket.onopen = function (event) {
    console.log("conntection established ...")
  };
  socket.onmessage = function (event) {
    var message = event.data;
    // console.log(message)
    var param = JSON.parse(message)
    console.log(param);
    switch (param.key) {
      case 2:
        $("#slider").slider("value", param.value);
        break;
      case 3:
        $("#checkbox-1").prop("checked", param.value).checkboxradio('refresh')
        break;
      case 4:
        $("#checkbox-2").prop("checked", param.value).checkboxradio('refresh')
        break;
    }
  };

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


  $( function() {
    $( "input" ).checkboxradio();
  } );

  $('#checkbox-1').click(function () {
    v = 0
    if ($('#checkbox-1').prop('checked')) {
        v = 1  
    }
    console.log("sending: " + v);
    var param = {
      key: 3,
      value: v
    };
    socket.send(JSON.stringify(param));
  });
  $('#checkbox-2').click(function () {
    v = 0
    if ($('#checkbox-2').prop('checked')) {
        v = 1  
    }
    console.log("sending: " + v);
    var param = {
      key: 4,
      value: v
    };
    socket.send(JSON.stringify(param));
  });


};
