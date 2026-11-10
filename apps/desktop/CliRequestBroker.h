#pragma once
class SingleInstance;
class UpdateController;
class ApplicationSessions;
void installCliRequests(SingleInstance *instance, UpdateController *updates,
                        ApplicationSessions *sessions);
