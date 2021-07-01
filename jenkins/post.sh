#!/bin/bash
if [ "$1" == "fixed" ]; then
    status='恢复'
elif [ "$1" == "regression" ]; then
    status='失败'
else
    exit 0
fi

curl -s -X POST https://hooks.slack.com/services/${webhook_key} \
    -H 'Content-Type: application/json; charset=utf-8' \
    --data-binary @- << EOF
{
    "channel": "#jenkins-notice",
    "username": "Jenkins Notification",
    "icon_emoji": ":four_leaf_clover:",
    "text": "[${JOB_NAME}] 分支构建 ${status}"
}
EOF
