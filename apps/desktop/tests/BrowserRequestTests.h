#pragma once

#include <QObject>

class BrowserRequestTests : public QObject {
    Q_OBJECT

  private slots:
    void pagedModelKeepsLogicalRowsAndSelectionWithinItsCacheBudget();
    void realLargeArchiveFetchesDistantRowsAndReplacesItsQuery();
    void taskAdmissionCountsOriginatingRequestOnce_data();
    void taskAdmissionCountsOriginatingRequestOnce();
    void unknownSizesRemainDistinctFromEmptyFilesAndNavigationSelection();
    void encryptedOpenAmbiguityReusesThePasswordPromptWithoutClaimingVerifierFailure();
    void mailboxBackpressureShutdownAndIdle();
};
