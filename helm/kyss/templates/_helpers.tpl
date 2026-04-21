{{/*
Common labels
*/}}
{{- define "kyss.labels" -}}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Values.global.imageTag | default .Chart.AppVersion | quote }}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "kyss.selectorLabels" -}}
app.kubernetes.io/name: kyss
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Full image reference
*/}}
{{- define "kyss.image" -}}
{{- if .Values.global.imageRegistry -}}
{{ .Values.global.imageRegistry }}/{{ .Values.kyss.image }}:{{ .Values.global.imageTag | default .Chart.AppVersion }}
{{- else -}}
{{ .Values.kyss.image }}:{{ .Values.global.imageTag | default .Chart.AppVersion }}
{{- end -}}
{{- end }}
