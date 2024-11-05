import QtQuick.Controls
import QtQuick.Window

Window {
    title: "Package Assistant"
    visible: true
    height: 480
    width: 640
    color: "#e4af79"

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        /* space between widget */
        spacing: 10

        Button {
            text: "I do nothing!"
        }
    }
}