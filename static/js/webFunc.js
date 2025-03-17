function getExtension(filename) {
    var parts = filename.split('.');
    return parts[parts.length - 1];
}

function failValidation(msg) {
    alert(msg); // just an alert for now but you can spice this up later
    return false;
}

function verify() {

}

//載入WebATM
async function LoadAtmUtility(handle,readerName,cardID) {
    if (!(await LoadWasm())) {
		alert("元件載入失敗，請重新整理網頁，並再試一次!!");
        return;
    }
    if (!(await WsConnect())) {
        //alert("元件連線失敗，請重新整理網頁，並再試一次!!");
        return;
    }

    await CheckServiceVersion();

    if (handle) {

        if ((!await XCsp.SetCardHandle(handle))) {
            ShowAtmUtilityErrorMessage();
            return;
        }
    }

    if (readerName) {
        if (!await XCsp.SetReaderName(readerName)) {
            ShowAtmUtilityErrorMessage();
            return;
        }
    }

    if (cardID) {
        if (await IsChangedCard(cardID)) {
            ShowAtmUtilityErrorMessage();
            return;
        }
    }
}


//連接服務
async function WsConnect() {
    if (!(await ConnectHandler.WsConnect())) {
        var isFirefox = navigator.userAgent.toLowerCase().indexOf('firefox') > -1;
        var isMacOS = (/macintosh|mac os x|macOS/i.test(navigator.userAgent));
        var msg ='';
        var checkisAPI_text = "";
        var installed_text = "安裝完畢後，重新整理此頁面即可開始服務。";
        msg = "請檢查三信商業銀行晶片卡服務程式(CotaIcSvr)是否已安裝並啟動。"

        alert(msg);
        return false;
    }

//設定為簡易模式(跳出來視窗操作時間會比較長)
    if (!(await XCsp.UseEasyMode(false))) {
            ShowAtmUtilityErrorMessage();
            return false;
        }
    return true;
}

//檢查服務版本
async function CheckServiceVersion() {

    var version = GetMatchingVersion();
    var serviceVersion = await XCsp.GetVersion();
    if (!version) {
        alert("取得版本失敗。");
        return;
    } else if (version != serviceVersion) {
        var isMacOS = (/macintosh|mac os x|macOS/i.test(navigator.userAgent));
        var msg = "";
        /*if (isMacOS) {
            msg = "<a id='alert-focus' href='#' role='alert' class='focusInModel' style='line-height:4rem'>服務程式版本不符，請重新下載<strong>三信商業銀行晶片卡服務程式(CotaWebATMService)</strong><br> </a><a class='btn btn-primary' href='/NewWebATM/Account/DownloadCOTABankWebATM?DownType=pkg'><span class='glyphicon glyphicon-download-alt'></span> 點此下載</a>";
        }
        else
        {
            msg = "<a id='alert-focus' href='#' role='alert' class='focusInModel' style='line-height:4rem'>服務程式版本不符，請重新下載<strong>三信商業銀行晶片卡服務程式(CotaICSvr)</strong><br> </a><a class='btn btn-primary' href='/NewWebATM/Account/DownloadCOTABankWebATM?DownType=exe'><span class='glyphicon glyphicon-download-alt'></span> 點此下載</a>";
        }*/
        msg = "服務程式版本不符，請重新安裝三信商業銀行晶片卡服務程式(CotaICSvr)";
        alert(msg);
        return;
    } else {
        
        await ListReaders();
    }
    return;
}

async function ConnectReader() {
    if (XCsp.isWaiting) {
        return;
    }
    var isConn = await XCsp.ConnectReader();
    if (!isConn) {
        alert("ConnectReader failed");
    }
}

//列出讀卡機
async function ListReaders() {
    if (XCsp.isWaiting) {
        return;
    }
    $("#addButton").text("讀取中");
    //$("#readingLabel").show();
    var readerNames = await XCsp.ListReaders();
    if (readerNames) {
        console.log(readerNames.toString());
        var readerSelect = $('#readerSelect');
        readerSelect.find('option').remove();
        if (readerSelect[0] != null) {
            if (readerSelect[0].nodeName === 'SELECT') {
                for (var i = 0; i < readerNames.length; i++) {
                    readerSelect.append(('<option>' + readerNames[i] + '</option>'));
                }
            }
        }
    }
    else
    {
        ShowAtmUtilityErrorMessage();
    }

    $("#addButton").text("重新讀取");
}

//讀取卡片

async function ConnectCard() {
    if (XCsp.isWaiting) {
        return false;
    }

    if (!await XCsp.ConnectCard()) {
        return false;
    }

    return await XCsp.ValidFiscCard();

}

async function ListOutAccountsOnCard($outAccountSelect) {
    var accountOutSelect;
    if (!$outAccountSelect) {
        //Default
        accountOutSelect = $("#OutAccountSelect");
    } else {
        accountOutSelect = $outAccountSelect;
    }
    accountOutSelect.append($('<option>', {
        value: ''
    })
		.text("轉出帳號讀取中...."));

    try {
        //沒設定會出錯
        var bankID = window.CurrentBankID;
        var bankName = window.CurrentBankName;

        while (XCsp.isWaiting) {
            await sleep(200);
        }

        //因為登入後讀取ATM元件有一連串的動作

        //跟讀取轉出帳號是非同步進行的

        //如果讀取卡片失敗的話，這個FUNCTION也會繼續做

        //繼續做的話就會讀取的時候也出錯誤訊息，這個FUNCTION也會出訊息

        //為了避免這樣的情況用讀取卡片ID來確認卡片是否還連著
        var cardID = await XCsp.ReadBinary();
        if(!cardID)
            return;

        var accountsOut = await XCsp.FiscListAccounts();

        accountOutSelect.find('option').remove();
        if (accountsOut) {
            var validAccountOut = 0;
            for (var i in accountsOut) {
                if (!accountsOut[i].match(/^\d{16}$/))
                    continue;
                if (accountsOut[i] === '0000000000000000')
                    continue;
                var selectKey = accountsOut[i];
                var selectValue = bankID + ' ' + bankName + ' ' + accountsOut[i];
                accountOutSelect.append($('<option>', {
                    value: selectKey
                })
					.text(selectValue));
                validAccountOut++;
            }

            if (validAccountOut == 1) {
                accountOutSelect.prop('disabled', 'disabled');
            }
        } else {
            accountOutSelect.append(('<option>此卡片上沒有任何轉出帳號</option>'));
            ShowAtmUtilityErrorMessage();
        }

    } catch (e) {
        accountOutSelect.find('option').remove();
        alert("取得晶片卡轉出帳號失敗!");
    } finally {
        // await XCsp.DisConnectCard();
    }
}

async function GetLastError() {
    var msg = await XCsp.GetLastError();
    return msg; 
}

function setCookie(cname,cvalue,exdays)
{
  var d = new Date();
  d.setTime(d.getTime()+(exdays*24*60*60*1000));
  var expires = "expires="+d.toGMTString();
  document.cookie = cname + "=" + cvalue + "; " + expires;
}

function getCookie(cname)
{
  var name = cname + "=";
  var ca = document.cookie.split(';');
  for(var i=0; i<ca.length; i++) 
  {
    var c = ca[i].trim();
    if (c.indexOf(name)==0) return c.substring(name.length,c.length);
  }
  return "";
}

async function ShowAtmUtilityErrorMessage()
{
    var errMsg = await GetLastError();
    alert(errMsg);
}