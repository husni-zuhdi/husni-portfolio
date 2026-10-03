# Use Google Cloud Storage for application secrets

Load sensitive application settings from a Google Cloud Storage object rather than
storing them as individual Google Secret Manager entries, because Google Secret
Manager was too expensive for this project. This record covers the application's
current GCS integration; infrastructure-level secret-manager choices are outside
its scope.
